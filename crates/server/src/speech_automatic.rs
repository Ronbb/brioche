//! Owner-authorized automatic assembly. No human hearing or timing decisions are inserted.
use crate::{
    AppError,
    identity::Backend,
    learning::{exec, field, hash, one},
};
use brioche_course_contract::{AdminAlignmentWord, AdminSpeechPackageRequest};
use sea_orm::{ConnectionTrait, IsolationLevel, TransactionTrait};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::PathBuf};

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, AppError> {
    value[key].as_str().ok_or(AppError::InvalidInput)
}
fn check_report(report: &Value, request: &AdminSpeechPackageRequest) -> Result<(), AppError> {
    crate::speech_package::settings(request)?;
    let model: Value = serde_json::from_str(include_str!("../../../scripts/alignment/model.json"))
        .map_err(|_| AppError::Unavailable)?;
    let versions: Value =
        serde_json::from_str(include_str!("../../../scripts/alignment/runtime.json"))
            .map_err(|_| AppError::Unavailable)?;
    let engine = &report["engine"];
    if serde_json::to_vec(report)
        .map_err(|_| AppError::InvalidInput)?
        .len()
        > 4 * 1024 * 1024
        || report["schemaVersion"] != "1.0"
        || report["kind"] != "brioche-automatic-alignment-v1"
        || report["reviewRequired"] != false
        || report["humanListeningAsserted"] != false
        || hash(report)? != request.expected_report_hash
        || !crate::voice_references::hex(text(report, "planId")?, 32)
        || !crate::voice_references::hex(text(report, "sourceArchiveSha256")?, 64)
        || !crate::voice_references::hex(text(report, "originalPredictionReportSha256")?, 64)
        || engine.as_object().is_none_or(|v| v.len() != 8)
        || engine["repository"] != model["repository"]
        || engine["revision"] != model["revision"]
        || engine["files"] != model["files"]
        || engine["versions"] != versions
        || engine["device"] != "cpu"
        || engine["dtype"] != "float32"
        || engine["attention"] != "eager"
        || engine["transcript"]
            != "NFC source word units, apostrophes normalized; original scalar ranges retained; raw timestamp classes without interpolation"
    {
        return Err(AppError::InvalidInput);
    }
    Ok(())
}
async fn snapshot(
    db: &impl ConnectionTrait,
    report: &Value,
    request: &AdminSpeechPackageRequest,
) -> Result<Value, AppError> {
    let input = crate::speech_export::snapshot_direct(db, text(report, "planId")?).await?;
    let plan = &input["plan"];
    if plan["planHash"] != report["planHash"] {
        return Err(AppError::Conflict);
    }
    let lesson = text(plan, "lessonId")?;
    let revision = plan["lessonRevision"]
        .as_u64()
        .and_then(|n| i32::try_from(n).ok())
        .ok_or(AppError::InvalidInput)?;
    let row = one(
        db,
        "SELECT server_document FROM lesson_revisions WHERE lesson_id=$1 AND revision=$2",
        vec![lesson.into(), revision.into()],
    )
    .await?
    .ok_or(AppError::NotFound)?;
    let source: Value = field(&row, "server_document")?;
    if hash(&source)? != plan["sourceHash"] {
        return Err(AppError::Conflict);
    }
    let latest = one(
        db,
        "SELECT MAX(revision) AS revision FROM lesson_revisions WHERE lesson_id=$1",
        vec![lesson.into()],
    )
    .await?
    .ok_or(AppError::Unavailable)?;
    if i64::from(request.lesson_revision) <= i64::from(field::<i32>(&latest, "revision")?) {
        return Err(AppError::Conflict);
    }
    Ok(json!({"input":input,"source":source}))
}
fn manifest(snapshot: &Value, report: &Value) -> Result<Value, AppError> {
    let input = &snapshot["input"];
    let originals = input["clips"].as_array().ok_or(AppError::Unavailable)?;
    let predictions = report["clips"].as_array().ok_or(AppError::InvalidInput)?;
    if predictions.len() != originals.len() || predictions.is_empty() || predictions.len() > 1000 {
        return Err(AppError::InvalidInput);
    }
    let mut seen = BTreeSet::new();
    let mut clips = Vec::new();
    for prediction in predictions {
        let key = text(prediction, "generationKey")?;
        if !seen.insert(key) || prediction["issues"] != json!([]) {
            return Err(AppError::InvalidInput);
        }
        let original = originals
            .iter()
            .find(|c| c["generationKey"] == key)
            .ok_or(AppError::Conflict)?;
        if original["id"] != prediction["clipId"]
            || original["result"]["sha256"] != prediction["sha256"]
            || original["result"]["durationMs"] != prediction["durationMs"]
        {
            return Err(AppError::Conflict);
        }
        let expected: Vec<AdminAlignmentWord> =
            serde_json::from_value(original["words"].clone()).map_err(|_| AppError::Unavailable)?;
        let words: Vec<AdminAlignmentWord> = serde_json::from_value(prediction["words"].clone())
            .map_err(|_| AppError::InvalidInput)?;
        let duration = prediction["durationMs"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or(AppError::InvalidInput)?;
        crate::speech_alignments::validate_words(&words, &expected, duration, true)?;
        let targets = prediction["targets"]
            .as_array()
            .ok_or(AppError::InvalidInput)?;
        let expected_targets: Vec<_> = input["plan"]["targets"]
            .as_array()
            .ok_or(AppError::Unavailable)?
            .iter()
            .filter(|t| t["generationKey"] == key)
            .collect();
        if targets.len() != expected_targets.len() {
            return Err(AppError::InvalidInput);
        }
        let mut pointers = BTreeSet::new();
        for target in targets {
            let pointer = text(target, "pointer")?;
            if !pointers.insert(pointer) || target["issues"] != json!([]) {
                return Err(AppError::InvalidInput);
            }
            let expected = expected_targets
                .iter()
                .find(|t| t["pointer"] == pointer)
                .ok_or(AppError::InvalidInput)?;
            if expected["blockId"] != target["blockId"] || expected["entryId"] != target["entryId"]
            {
                return Err(AppError::InvalidInput);
            }
            let mut mapped = Vec::new();
            for unit in expected["words"].as_array().ok_or(AppError::Unavailable)? {
                let word = words
                    .iter()
                    .find(|w| {
                        json!(w.start) == unit["entryStart"]
                            && json!(w.end) == unit["entryEnd"]
                            && json!(w.text) == unit["text"]
                    })
                    .ok_or(AppError::InvalidInput)?;
                let mut value = unit.clone();
                value["startMs"] = json!(word.start_ms);
                value["endMs"] = json!(word.end_ms);
                mapped.push(value);
            }
            if json!(mapped) != target["words"] {
                return Err(AppError::InvalidInput);
            }
        }
        clips.push(json!({"id":original["id"],"generationKey":key,"result":original["result"],"words":words,"review":null,"speechReview":null}));
    }
    let report_hash = hash(report)?;
    Ok(
        json!({"schemaVersion":"1.0","kind":"brioche-automatic-speech-package","alignmentId":format!("automatic-{report_hash}"),"reportHash":report_hash,"predictionEngine":report["engine"],"planId":input["planId"],"plan":input["plan"],"source":snapshot["source"],"clips":clips,"automaticAlignment":report,"humanListeningAsserted":false}),
    )
}
/// Private archive only. Registration, version import and directory activation remain explicit.
pub async fn assemble_for_actor(
    b: &Backend,
    actor: i64,
    root: PathBuf,
    report: Value,
    request: AdminSpeechPackageRequest,
) -> Result<Vec<u8>, AppError> {
    check_report(&report, &request)?;
    let tx =
        b.db.begin_with_config(Some(IsolationLevel::RepeatableRead), None)
            .await
            .map_err(|_| AppError::Unavailable)?;
    crate::voice_references::lock_operator(&tx, actor).await?;
    let original = snapshot(&tx, &report, &request).await?;
    tx.commit().await.map_err(|_| AppError::Unavailable)?;
    let expected = original.clone();
    let assembled_manifest = manifest(&original, &report)?;
    let config = request.clone();
    let archive_sha = text(&report, "sourceArchiveSha256")?.to_owned();
    let bytes = tokio::task::spawn_blocking(move || {
        let input = crate::speech_export::pack(&root, original["input"].clone())?;
        if crate::media::digest(&input) != archive_sha {
            return Err(AppError::Conflict);
        }
        crate::speech_package::pack_automatic(&root, assembled_manifest, &config, actor)
    })
    .await
    .map_err(|_| AppError::Unavailable)??;
    let tx = b.db.begin().await.map_err(|_| AppError::Unavailable)?;
    crate::voice_references::lock_operator(&tx, actor).await?;
    exec(
        &tx,
        "SELECT singleton FROM content_state WHERE singleton FOR UPDATE",
        vec![],
    )
    .await?;
    if snapshot(&tx, &report, &request).await? != expected {
        return Err(AppError::Conflict);
    }
    tx.commit().await.map_err(|_| AppError::Unavailable)?;
    Ok(bytes)
}
