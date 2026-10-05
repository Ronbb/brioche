//! Immutable recording registry. Registration does not make a recording public.
use crate::{
    audio,
    learning::{exec, field, hash, one},
    media,
};
use anyhow::{Context, Result, ensure};
use brioche_course_contract::{AudioAsset, PublicLesson};
use sea_orm::{ConnectionTrait, DatabaseConnection, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::{Component, Path},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioBundle {
    pub schema_version: String,
    pub assets: Vec<AudioSpec>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioSpec {
    pub asset_id: String,
    pub revision: u32,
    pub sha256: String,
    pub mime_type: String,
    pub duration_ms: u32,
    pub credit_zh: String,
    pub file: String,
    pub status: String,
    pub source: String,
    pub license: String,
    pub creator: String,
    pub rights_confirmed: bool,
}

impl AudioBundle {
    fn validate(&self, actor: &str) -> Result<()> {
        ensure!(
            self.schema_version == "1.0"
                && (1..=500).contains(&self.assets.len())
                && media::text(actor),
            "invalid recording bundle or actor"
        );
        let mut ids = BTreeSet::new();
        for spec in &self.assets {
            ensure!(
                media::valid_id(&spec.asset_id)
                    && (1..=i32::MAX as u32).contains(&spec.revision)
                    && ids.insert((&spec.asset_id, spec.revision)),
                "invalid or duplicate recording identity"
            );
            ensure!(
                spec.status == "ready"
                    && spec.rights_confirmed
                    && [&spec.source, &spec.license, &spec.creator, &spec.credit_zh]
                        .into_iter()
                        .all(|value| media::text(value)),
                "recording requires ready status, confirmed rights, source, license, creator and credit"
            );
            ensure!(
                spec.sha256.len() == 64
                    && spec
                        .sha256
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
                "recording SHA-256 must be lowercase hex"
            );
            ensure!(
                (1..=1_800_000).contains(&spec.duration_ms),
                "recording duration out of range"
            );
            audio::extension(&spec.mime_type)?;
            ensure!(
                !spec.file.is_empty()
                    && Path::new(&spec.file)
                        .components()
                        .all(|c| matches!(c, Component::Normal(_))),
                "recording path must be relative without traversal"
            );
        }
        Ok(())
    }
}

pub async fn import_bundle(
    db: &DatabaseConnection,
    bundle: AudioBundle,
    source_root: &Path,
    store: &Path,
    actor: &str,
) -> Result<()> {
    bundle.validate(actor)?;
    let bundle_hash = hash(&bundle).map_err(anyhow::Error::msg)?;
    let source_root = source_root.canonicalize()?;
    let store = store.to_path_buf();
    let specs = bundle.assets.clone();
    let recordings = tokio::task::spawn_blocking(move || -> Result<Vec<_>> {
        let mut result = Vec::new();
        for spec in specs {
            let path = source_root.join(&spec.file).canonicalize()?;
            ensure!(
                path.starts_with(&source_root),
                "recording escapes source directory"
            );
            let (bytes, info) = audio::inspect_file(&path, &spec.mime_type)?;
            ensure!(
                media::digest(&bytes) == spec.sha256,
                "recording hash mismatch"
            );
            ensure!(
                info.duration_ms == spec.duration_ms,
                "recording duration does not match decoded frames"
            );
            let ext = audio::extension(&spec.mime_type)?;
            media::store_file(&store, &bytes, &spec.sha256, ext)?;
            let descriptor = AudioAsset {
                asset_id: spec.asset_id.clone(),
                revision: spec.revision,
                sha256: spec.sha256.clone(),
                mime_type: spec.mime_type.clone(),
                duration_ms: info.duration_ms,
                credit_zh: spec.credit_zh.clone(),
                url: format!("/api/audio/{}.{}", spec.sha256, ext),
            };
            result.push((spec, descriptor, bytes.len(), info));
        }
        Ok(result)
    })
    .await??;
    let tx = db.begin().await?;
    one(
        &tx,
        "SELECT generation FROM content_state WHERE singleton FOR UPDATE",
        vec![],
    )
    .await
    .map_err(anyhow::Error::msg)?
    .context("content state missing")?;
    for (spec, descriptor, size, info) in recordings {
        exec(&tx, "INSERT INTO audio_assets(asset_id,revision,descriptor,provenance,sha256,extension,byte_size,duration_ms,sample_rate,channels) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)", vec![
            spec.asset_id.clone().into(), (spec.revision as i32).into(), serde_json::to_value(descriptor)?.into(),
            serde_json::to_value(&spec)?.into(), spec.sha256.into(), audio::extension(&spec.mime_type)?.into(),
            (size as i64).into(), (info.duration_ms as i32).into(), (info.sample_rate as i32).into(), (info.channels as i32).into(),
        ]).await.map_err(anyhow::Error::msg)?;
    }
    exec(
        &tx,
        "INSERT INTO audio_import_audit(actor,bundle_hash,asset_count) VALUES($1,$2,$3)",
        vec![
            actor.into(),
            bundle_hash.into(),
            (bundle.assets.len() as i32).into(),
        ],
    )
    .await
    .map_err(anyhow::Error::msg)?;
    tx.commit().await?;
    Ok(())
}

pub fn source_audio_refs(source: &serde_json::Value) -> Result<Vec<media::AssetRef>> {
    media::source_refs(source, "audioRefs")
}

pub async fn hydrate_source<C: ConnectionTrait>(
    db: &C,
    mut source: serde_json::Value,
) -> Result<serde_json::Value> {
    let refs = source_audio_refs(&source)?;
    if source.get("audioRefs").is_none() {
        return Ok(source);
    }
    let mut descriptors = Vec::new();
    for reference in refs {
        let row = one(
            db,
            "SELECT descriptor FROM audio_assets WHERE asset_id=$1 AND revision=$2",
            vec![
                reference.asset_id.into(),
                (reference.revision as i32).into(),
            ],
        )
        .await
        .map_err(anyhow::Error::msg)?
        .context("registered recording revision missing")?;
        descriptors
            .push(field::<serde_json::Value>(&row, "descriptor").map_err(anyhow::Error::msg)?);
    }
    source["audio"] = serde_json::Value::Array(descriptors);
    Ok(source)
}

/// Rechecks immutable registration and stored bytes before staging/activating a release.
pub async fn validate_lesson<C: ConnectionTrait>(
    db: &C,
    lesson: &PublicLesson,
    root: &Path,
) -> Result<(), crate::AppError> {
    for asset in &lesson.audio {
        let row = one(db, "SELECT descriptor,provenance,byte_size,sample_rate,channels FROM audio_assets WHERE asset_id=$1 AND revision=$2", vec![asset.asset_id.clone().into(), (asset.revision as i32).into()]).await?.ok_or(crate::AppError::InvalidInput)?;
        if field::<serde_json::Value>(&row, "descriptor")?
            != serde_json::to_value(asset).map_err(|_| crate::AppError::Unavailable)?
        {
            return Err(crate::AppError::InvalidInput);
        }
        let spec: AudioSpec = serde_json::from_value(field(&row, "provenance")?)
            .map_err(|_| crate::AppError::Unavailable)?;
        AudioBundle {
            schema_version: "1.0".into(),
            assets: vec![spec],
        }
        .validate("publication-validation")
        .map_err(|_| crate::AppError::InvalidInput)?;
        let expected_size = field::<i64>(&row, "byte_size")?;
        let expected_rate = field::<i32>(&row, "sample_rate")?;
        let expected_channels = field::<i32>(&row, "channels")?;
        let root = root.to_path_buf();
        let asset = asset.clone();
        tokio::task::spawn_blocking(move || -> Result<()> {
            let bytes =
                media::stored_bytes(&root, &asset.sha256, audio::extension(&asset.mime_type)?)?;
            ensure!(
                media::digest(&bytes) == asset.sha256 && bytes.len() as i64 == expected_size,
                "registered recording bytes mismatch"
            );
            let info = audio::inspect(&bytes, &asset.mime_type)?;
            ensure!(
                info.duration_ms == asset.duration_ms
                    && info.sample_rate as i32 == expected_rate
                    && info.channels as i32 == expected_channels,
                "registered recording decode mismatch"
            );
            Ok(())
        })
        .await
        .map_err(|_| crate::AppError::Unavailable)?
        .map_err(|_| crate::AppError::InvalidInput)?;
    }
    Ok(())
}

#[derive(Clone)]
struct AudioState {
    db: DatabaseConnection,
    root: std::path::PathBuf,
    permits: std::sync::Arc<tokio::sync::Semaphore>,
}
pub fn router(db: DatabaseConnection, root: std::path::PathBuf) -> axum::Router {
    axum::Router::new()
        .route("/api/audio/{name}", axum::routing::get(serve))
        .with_state(AudioState {
            db,
            root,
            permits: std::sync::Arc::new(tokio::sync::Semaphore::new(2)),
        })
}
async fn serve(
    axum::extract::State(state): axum::extract::State<AudioState>,
    axum::extract::Path(name): axum::extract::Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<axum::response::Response, crate::AppError> {
    let (sha, ext) = name.split_once('.').ok_or(crate::AppError::NotFound)?;
    if sha.len() != 64
        || !sha
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || !matches!(ext, "mp3" | "wav")
    {
        return Err(crate::AppError::NotFound);
    }
    let row = one(&state.db, "SELECT descriptor FROM audio_assets a WHERE sha256=$1 AND extension=$2 AND EXISTS(SELECT 1 FROM lesson_revisions r WHERE r.published AND NOT EXISTS(SELECT 1 FROM content_withdrawals w WHERE (w.lesson_id,w.revision)=(r.lesson_id,r.revision)) AND r.public_document->'audio' @> jsonb_build_array(jsonb_build_object('assetId',a.asset_id,'revision',a.revision))) LIMIT 1", vec![sha.into(),ext.into()]).await?.ok_or(crate::AppError::NotFound)?;
    let descriptor = serde_json::from_value(field(&row, "descriptor")?)
        .map_err(|_| crate::AppError::Unavailable)?;
    asset_response(state.root, descriptor, state.permits, headers).await
}

pub(crate) async fn asset_response(
    root: std::path::PathBuf,
    descriptor: AudioAsset,
    permits: std::sync::Arc<tokio::sync::Semaphore>,
    headers: axum::http::HeaderMap,
) -> Result<axum::response::Response, crate::AppError> {
    use axum::{
        body::Body,
        http::{StatusCode, header},
        response::Response,
    };
    let ext = audio::extension(&descriptor.mime_type)
        .map_err(|_| crate::AppError::Unavailable)?
        .to_owned();
    let sha = descriptor.sha256;
    if sha.len() != 64
        || !sha
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(crate::AppError::Unavailable);
    }
    let etag = format!("\"{sha}\"");
    let permit = permits
        .try_acquire_owned()
        .map_err(|_| crate::AppError::Unavailable)?;
    let bytes = tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
        let _permit = permit;
        let bytes = media::stored_bytes(&root, &sha, &ext)?;
        ensure!(media::digest(&bytes) == sha, "recording object corrupt");
        Ok(bytes)
    })
    .await
    .map_err(|_| crate::AppError::Unavailable)?
    .map_err(|_| crate::AppError::Unavailable)?;
    let size = bytes.len();
    let mut builder = Response::builder()
        .header(header::CONTENT_TYPE, descriptor.mime_type)
        .header(header::CACHE_CONTROL, "no-store")
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::ETAG, &etag)
        .header("x-content-type-options", "nosniff")
        .header("cross-origin-resource-policy", "same-origin");
    let mut span = 0..size;
    if headers.contains_key(header::RANGE)
        && headers
            .get(header::IF_RANGE)
            .is_none_or(|value| value.as_bytes() == etag.as_bytes())
    {
        let values: Vec<_> = headers.get_all(header::RANGE).iter().collect();
        let range = if values.len() == 1 {
            values[0]
                .to_str()
                .ok()
                .and_then(|value| byte_range(value, size))
        } else {
            None
        };
        let Some(range) = range else {
            return builder
                .status(StatusCode::RANGE_NOT_SATISFIABLE)
                .header(header::CONTENT_RANGE, format!("bytes */{size}"))
                .header(header::CONTENT_LENGTH, 0)
                .body(Body::empty())
                .map_err(|_| crate::AppError::Unavailable);
        };
        span = range;
        builder = builder.status(StatusCode::PARTIAL_CONTENT).header(
            header::CONTENT_RANGE,
            format!("bytes {}-{}/{size}", span.start, span.end - 1),
        );
    }
    let length = span.len();
    let body = if span.start == 0 && span.end == size {
        bytes
    } else {
        bytes[span].to_vec()
    };
    builder
        .header(header::CONTENT_LENGTH, length)
        .body(Body::from(body))
        .map_err(|_| crate::AppError::Unavailable)
}

fn byte_range(value: &str, size: usize) -> Option<std::ops::Range<usize>> {
    if value.len() > 100 || size == 0 {
        return None;
    }
    let value = value.strip_prefix("bytes=")?;
    let (start, end) = value.split_once('-')?;
    let number = |text: &str| -> Option<usize> {
        if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
            None
        } else {
            text.parse().ok()
        }
    };
    if start.is_empty() {
        let suffix = number(end)?;
        return (suffix > 0).then_some(size.saturating_sub(suffix)..size);
    }
    let start = number(start)?;
    if start >= size {
        return None;
    }
    let end = if end.is_empty() {
        size - 1
    } else {
        number(end)?.min(size - 1)
    };
    (end >= start).then_some(start..end + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn single_byte_ranges_are_bounded() {
        assert_eq!(byte_range("bytes=2-5", 10), Some(2..6));
        assert_eq!(byte_range("bytes=2-", 10), Some(2..10));
        assert_eq!(byte_range("bytes=-3", 10), Some(7..10));
        assert_eq!(byte_range("bytes=0-999", 10), Some(0..10));
        assert_eq!(byte_range("bytes=-999", 10), Some(0..10));
        for value in [
            "bytes=10-",
            "bytes=5-2",
            "bytes=-0",
            "bytes=",
            "bytes=1-2,3-4",
            "items=1-2",
            "bytes=+1-2",
            "bytes=0-999999999999999999999999999999",
        ] {
            assert_eq!(byte_range(value, 10), None, "{value}");
        }
    }
    #[test]
    fn refuses_missing_rights_and_bad_references() {
        for value in [
            serde_json::json!({"audioRefs":null}),
            serde_json::json!({"audioRefs":[{"assetId":"audio","revision":0}]}),
            serde_json::json!({"audioRefs":[{"assetId":"audio","revision":1},{"assetId":"audio","revision":2}]}),
            serde_json::json!({"audioRefs":[{"assetId":"audio","revision":1,"ignored":true}]}),
        ] {
            assert!(source_audio_refs(&value).is_err());
        }
        let mut bundle = AudioBundle {
            schema_version: "1.0".into(),
            assets: vec![AudioSpec {
                asset_id: "original".into(),
                revision: 1,
                sha256: "0".repeat(64),
                mime_type: "audio/mpeg".into(),
                duration_ms: 1000,
                credit_zh: "原创测试音".into(),
                file: "synthetic.mp3".into(),
                status: "ready".into(),
                source: "local synthesis".into(),
                license: "original protocol fixture".into(),
                creator: "test generator".into(),
                rights_confirmed: true,
            }],
        };
        assert!(bundle.validate("operator").is_ok());
        bundle.assets[0].rights_confirmed = false;
        assert!(bundle.validate("operator").is_err());
        bundle.assets[0].rights_confirmed = true;
        bundle.assets[0].file = "../escape.mp3".into();
        assert!(bundle.validate("operator").is_err());
    }
}
