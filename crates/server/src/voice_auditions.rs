//! Audition attempts are durable before the paid call. Files remain private and reviews append versions.
use crate::{
    AppError,
    identity::{AuthSession, Backend, require_operator},
    learning::{exec, field, one, owner},
    qwen::{ProviderError, Service},
    voice_references::{hex, lock_operator},
};
use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
};
use brioche_course_contract::{
    AdminAudition, AdminAuditionRequest, AdminAuditionReview, AdminAuditions,
    AdminCharacterVoiceRequest, CharacterVoiceProfile, VoiceJobStatus,
};
use sea_orm::{ConnectionTrait, DbBackend, QueryResult, Statement, TransactionTrait};
use serde_json::{Value, json};
use std::{path::PathBuf, sync::Arc};
const SELECT: &str = r#"SELECT a.id,a.clone_job_id,g.character_id,g.character_revision,g.voice_revision,a.profile,a.parameters,e.result,
CASE WHEN e.status='submitted' AND e.created_at<clock_timestamp()-interval '300 seconds' THEN 'unknown' ELSE e.status END AS status,
r.accepted,r.voice_revision AS applied_voice_revision,to_char(a.created_at AT TIME ZONE 'UTC','YYYY-MM-DD"T"HH24:MI:SS.US"Z"') AS created_at
FROM voice_auditions a JOIN voice_clone_jobs j ON j.id=a.clone_job_id JOIN voice_reference_grants g ON g.id=j.grant_id
JOIN LATERAL (SELECT * FROM voice_audition_events WHERE audition_id=a.id ORDER BY version DESC LIMIT 1) e ON true
LEFT JOIN voice_audition_reviews r ON r.audition_id=a.id"#;
pub fn router() -> Router<Backend> {
    Router::new()
        .route("/api/v1/operator/voice-auditions", get(list).post(create))
        .route("/api/v1/operator/voice-auditions/{id}", get(read))
        .route("/api/v1/operator/voice-auditions/{id}/file", get(file))
        .route("/api/v1/operator/voice-auditions/{id}/review", post(review))
}
fn item(row: &QueryResult) -> Result<AdminAudition, AppError> {
    let p: Value = field(row, "profile")?;
    let parameters: Value = field(row, "parameters")?;
    let result: Option<Value> = field(row, "result")?;
    Ok(AdminAudition {
        id: field(row, "id")?,
        clone_job_id: field(row, "clone_job_id")?,
        character_id: field(row, "character_id")?,
        character_revision: field::<i32>(row, "character_revision")? as u32,
        base_voice_revision: field::<i32>(row, "voice_revision")? as u32,
        voice_id: p["voiceId"].as_str().ok_or(AppError::Unavailable)?.into(),
        text: parameters["input"]["text"]
            .as_str()
            .ok_or(AppError::Unavailable)?
            .into(),
        emotion: parameters["sceneEmotion"]
            .as_str()
            .ok_or(AppError::Unavailable)?
            .into(),
        status: field(row, "status")?,
        duration_ms: result
            .as_ref()
            .and_then(|r| r["durationMs"].as_u64())
            .map(|n| n as u32),
        request_id: result
            .as_ref()
            .and_then(|r| r["requestId"].as_str())
            .map(String::from),
        accepted: field(row, "accepted")?,
        applied_voice_revision: field::<Option<i32>>(row, "applied_voice_revision")?
            .map(|n| n as u32),
        created_at: field(row, "created_at")?,
    })
}
async fn load(db: &impl ConnectionTrait, id: &str) -> Result<QueryResult, AppError> {
    if !hex(id, 32) {
        return Err(AppError::InvalidInput);
    }
    one(db, &format!("{SELECT} WHERE a.id=$1"), vec![id.into()])
        .await?
        .ok_or(AppError::NotFound)
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Cursor {
    after_id: Option<String>,
    clone_job_id: Option<String>,
}
async fn list(
    auth: AuthSession,
    State(b): State<Backend>,
    service: Option<Extension<Service>>,
    Query(cursor): Query<Cursor>,
) -> Result<Json<AdminAuditions>, AppError> {
    require_operator(&auth)?;
    let after = cursor.after_id.unwrap_or_default();
    let clone = cursor.clone_job_id;
    if (!after.is_empty() && !hex(&after, 32)) || clone.as_ref().is_some_and(|s| !hex(s, 32)) {
        return Err(AppError::InvalidInput);
    }
    let rows=b.db.query_all_raw(Statement::from_sql_and_values(DbBackend::Postgres,format!("{SELECT} WHERE a.id>$1 AND ($2::text IS NULL OR a.clone_job_id=$2) ORDER BY a.id LIMIT 21"),vec![after.into(),clone.into()])).await.map_err(|_|AppError::Unavailable)?;
    let items = rows
        .iter()
        .take(20)
        .map(item)
        .collect::<Result<Vec<_>, _>>()?;
    let next = if rows.len() > 20 {
        items.last().map(|i| i.id.clone())
    } else {
        None
    };
    Ok(Json(AdminAuditions {
        items,
        next,
        configured: service.is_some(),
    }))
}
async fn read(
    auth: AuthSession,
    State(b): State<Backend>,
    Path(id): Path<String>,
) -> Result<Json<AdminAudition>, AppError> {
    require_operator(&auth)?;
    Ok(Json(item(&load(&b.db, &id).await?)?))
}
async fn create(
    auth: AuthSession,
    State(b): State<Backend>,
    service: Option<Extension<Service>>,
    Extension(root): Extension<PathBuf>,
    Json(request): Json<AdminAuditionRequest>,
) -> Result<Json<AdminAudition>, AppError> {
    require_operator(&auth)?;
    crate::admin::reason(&request.reason)?;
    if !request.cost_confirmed
        || !hex(&request.id, 32)
        || !hex(&request.clone_job_id, 32)
        || request.expected_clone_version == 0
        || request.expected_clone_version > i32::MAX as u32
    {
        return Err(AppError::InvalidInput);
    }
    let actor = owner(&auth)?;
    let tx = b.db.begin().await.map_err(|_| AppError::Unavailable)?;
    lock_operator(&tx, actor).await?;
    // Serialize the id before any external call. Exact retries return the original attempt without resending.
    exec(
        &tx,
        "SELECT pg_advisory_xact_lock(hashtextextended($1,0))",
        vec![format!("voice-audition:{}", request.id).into()],
    )
    .await?;
    if let Some(row)=one(&tx,"SELECT actor_id,reason,clone_job_id,clone_version,parameters FROM voice_auditions WHERE id=$1",vec![request.id.clone().into()]).await?{
        let p:Value=field(&row,"parameters")?;
        if field::<i64>(&row,"actor_id")?!=actor||field::<String>(&row,"reason")?!=request.reason||field::<String>(&row,"clone_job_id")?!=request.clone_job_id||field::<i32>(&row,"clone_version")? as u32!=request.expected_clone_version||p["input"]["text"]!=request.text||p["sceneEmotion"]!=request.emotion{return Err(AppError::Conflict);}
        return Ok(Json(item(&load(&tx,&request.id).await?)?));
    }
    let service = service.ok_or(AppError::Unavailable)?.0;
    let permit = service
        .permits
        .clone()
        .try_acquire_owned()
        .map_err(|_| AppError::RateLimited)?;
    exec(
        &tx,
        "SELECT id FROM voice_clone_jobs WHERE id=$1 FOR UPDATE",
        vec![request.clone_job_id.clone().into()],
    )
    .await?;
    let source = crate::voice_jobs::load(&tx, &request.clone_job_id).await?;
    if source.version != request.expected_clone_version
        || !matches!(source.status, VoiceJobStatus::Ready)
    {
        return Err(AppError::Conflict);
    }
    let profile_row=one(&tx,"SELECT profile FROM character_voice_profiles WHERE character_id=$1 AND character_revision=$2 AND revision=$3",vec![source.character_id.into(),(source.character_revision as i32).into(),(source.voice_revision as i32).into()]).await?.ok_or(AppError::NotFound)?;
    let mut profile: CharacterVoiceProfile =
        serde_json::from_value(field(&profile_row, "profile")?)
            .map_err(|_| AppError::Unavailable)?;
    profile.voice_id = source.voice_id.ok_or(AppError::Conflict)?;
    profile.voice_kind = "cloned".into();
    profile.model = crate::qwen::MODEL.into();
    profile.provider = "qwen".into();
    let speech = crate::qwen::SpeechRequest {
        profile,
        text: request.text,
        emotion: request.emotion,
    };
    let mut parameters = speech.parameters().map_err(|_| AppError::InvalidInput)?;
    parameters["sceneEmotion"] = json!(speech.emotion); // Private reproducibility metadata, not sent to Qwen.
    exec(&tx,"INSERT INTO voice_auditions(id,clone_job_id,clone_version,profile,parameters,actor_id,reason) VALUES($1,$2,$3,$4,$5,$6,$7)",vec![request.id.clone().into(),request.clone_job_id.into(),(request.expected_clone_version as i32).into(),serde_json::to_value(&speech.profile).map_err(|_|AppError::InvalidInput)?.into(),parameters.into(),actor.into(),request.reason.into()]).await?;
    exec(
        &tx,
        "INSERT INTO voice_audition_events(audition_id,version,status) VALUES($1,1,'submitted')",
        vec![request.id.clone().into()],
    )
    .await?;
    let result = item(&load(&tx, &request.id).await?)?;
    tx.commit().await.map_err(|_| AppError::Unavailable)?;
    tokio::spawn(async move {
        let _permit = permit;
        let response = tokio::time::timeout(
            std::time::Duration::from_secs(240),
            service.transport.synthesize(&speech),
        )
        .await
        .unwrap_or(Err(ProviderError::Unknown));
        let (status, result) = match response {
            Ok(s) => match tokio::task::spawn_blocking(move || store(&root, s)).await {
                Ok(Ok(r)) => ("ready", Some(r)),
                _ => ("unknown", None),
            },
            Err(ProviderError::Rejected) => ("failed", None),
            Err(ProviderError::Unknown) => ("unknown", None),
        };
        if finish(&b.db, &request.id, status, result).await.is_err() {
            tracing::warn!("voice audition result persistence unavailable");
        }
    });
    Ok(Json(result))
}
fn store(root: &std::path::Path, s: crate::qwen::Speech) -> Result<Value, AppError> {
    let info = crate::audio::inspect(&s.wav, "audio/wav").map_err(|_| AppError::Unavailable)?;
    if s.provider_wav.len() < 44
        || !s.provider_wav.starts_with(b"RIFF")
        || &s.provider_wav[8..12] != b"WAVE"
        || s.provider_wav.len() > 16 * 1024 * 1024
        || s.wav.len() > 16 * 1024 * 1024
        || info.sample_rate != 24000
        || info.channels != 1
        || info.duration_ms > 180000
        || info != s.info
        || !crate::qwen::valid_id(&s.request_id)
        || s.verification.as_ref().is_none_or(|v| {
            v.model != crate::qwen::MODEL
                || v.status != "OK"
                || !crate::qwen::valid_id(&v.request_id)
        })
    {
        return Err(AppError::Unavailable);
    }
    let sha = crate::media::digest(&s.wav);
    let provider_sha = crate::media::digest(&s.provider_wav);
    crate::media::store_file(root, &s.provider_wav, &provider_sha, "wav")
        .map_err(|_| AppError::Unavailable)?;
    crate::media::store_file(root, &s.wav, &sha, "wav").map_err(|_| AppError::Unavailable)?;
    let verify = s.verification.unwrap();
    Ok(
        json!({"sha256":sha,"providerSha256":provider_sha,"byteLength":s.wav.len(),"durationMs":info.duration_ms,"requestId":s.request_id,"inputTokens":s.input_tokens,"outputTokens":s.output_tokens,"verification":{"model":verify.model,"status":verify.status,"requestId":verify.request_id},"postprocessing":"qwen-riff-length-v1-metadata-preserved"}),
    )
}
async fn finish(
    db: &impl ConnectionTrait,
    id: &str,
    status: &str,
    result: Option<Value>,
) -> Result<(), AppError> {
    exec(
        db,
        "INSERT INTO voice_audition_events(audition_id,version,status,result) VALUES($1,2,$2,$3)",
        vec![id.into(), status.into(), result.into()],
    )
    .await
    .map(|_| ())
}
async fn file(
    auth: AuthSession,
    State(b): State<Backend>,
    Path(id): Path<String>,
    Extension(root): Extension<PathBuf>,
    Extension(permits): Extension<Arc<tokio::sync::Semaphore>>,
    headers: HeaderMap,
) -> Result<axum::response::Response, AppError> {
    require_operator(&auth)?;
    let row = load(&b.db, &id).await?;
    if item(&row)?.status != "ready" {
        return Err(AppError::NotFound);
    }
    let result: Value = field::<Option<Value>>(&row, "result")?.ok_or(AppError::NotFound)?;
    let sha = result["sha256"]
        .as_str()
        .filter(|s| hex(s, 64))
        .ok_or(AppError::Unavailable)?
        .to_owned();
    let duration = result["durationMs"].as_u64().ok_or(AppError::Unavailable)?;
    let length = result["byteLength"].as_u64().ok_or(AppError::Unavailable)?;
    let _permit = permits
        .try_acquire_owned()
        .map_err(|_| AppError::RateLimited)?;
    let hash = sha.clone();
    let bytes = tokio::task::spawn_blocking(move || {
        let bytes =
            crate::media::stored_bytes(&root, &hash, "wav").map_err(|_| AppError::Unavailable)?;
        let info = crate::audio::inspect(&bytes, "audio/wav").map_err(|_| AppError::Unavailable)?;
        if crate::media::digest(&bytes) != hash
            || bytes.len() as u64 != length
            || info.duration_ms as u64 != duration
            || info.channels != 1
            || info.sample_rate != 24000
        {
            return Err(AppError::Unavailable);
        }
        Ok(bytes)
    })
    .await
    .map_err(|_| AppError::Unavailable)??;
    crate::recording::bytes_response("audio/wav".into(), format!("\"{sha}\""), bytes, headers)
}
async fn review(
    auth: AuthSession,
    State(b): State<Backend>,
    Path(id): Path<String>,
    Json(request): Json<AdminAuditionReview>,
) -> Result<Json<AdminAudition>, AppError> {
    require_operator(&auth)?;
    crate::admin::reason(&request.reason)?;
    if !hex(&id, 32) || !request.heard {
        return Err(AppError::InvalidInput);
    }
    let actor = owner(&auth)?;
    let tx = b.db.begin().await.map_err(|_| AppError::Unavailable)?;
    lock_operator(&tx, actor).await?;
    exec(
        &tx,
        "SELECT id FROM voice_auditions WHERE id=$1 FOR UPDATE",
        vec![id.clone().into()],
    )
    .await?;
    let row = load(&tx, &id).await?;
    let audition = item(&row)?;
    if audition.status != "ready"
        || audition.accepted.is_some()
        || request.expected_voice_revision != audition.base_voice_revision
    {
        return Err(AppError::Conflict);
    }
    let voice = if request.accepted {
        let profile =
            serde_json::from_value(field(&row, "profile")?).map_err(|_| AppError::Unavailable)?;
        Some(
            crate::character_voices::append_profile_in(
                &tx,
                actor,
                AdminCharacterVoiceRequest {
                    character_id: audition.character_id.clone(),
                    character_revision: audition.character_revision,
                    expected_voice_revision: request.expected_voice_revision,
                    profile,
                    reason: request.reason.clone(),
                },
            )
            .await?
            .voice_revision as i32,
        )
    } else {
        None
    };
    exec(&tx,"INSERT INTO voice_audition_reviews(audition_id,accepted,character_id,character_revision,voice_revision,actor_id,reason) VALUES($1,$2,$3,$4,$5,$6,$7)",vec![id.clone().into(),request.accepted.into(),audition.character_id.into(),(audition.character_revision as i32).into(),voice.into(),actor.into(),request.reason.into()]).await?;
    let result = item(&load(&tx, &id).await?)?;
    tx.commit().await.map_err(|_| AppError::Unavailable)?;
    Ok(Json(result))
}
