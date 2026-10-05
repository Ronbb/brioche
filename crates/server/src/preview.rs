//! Imported immutable revisions are visible only to an authenticated operator.
use crate::{
    AppError,
    identity::{AuthSession, Backend, require_operator},
    learning::{field, one},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use brioche_course_contract::PublicLesson;

#[derive(Clone)]
struct PreviewMedia {
    root: std::path::PathBuf,
    permits: std::sync::Arc<tokio::sync::Semaphore>,
}
pub fn router(root: std::path::PathBuf) -> Router<Backend> {
    Router::new()
        .route(
            "/api/v1/operator/lessons/{id}/revisions/{revision}",
            get(lesson),
        )
        .route(
            "/api/v1/operator/lessons/{id}/revisions/{revision}/media/{name}",
            get(media),
        )
        .layer(axum::Extension(PreviewMedia {
            root,
            permits: std::sync::Arc::new(tokio::sync::Semaphore::new(2)),
        }))
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 100
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}
async fn read(backend: &Backend, id: &str, revision: u32) -> Result<PublicLesson, AppError> {
    if !valid_id(id) || revision == 0 || revision > i32::MAX as u32 {
        return Err(AppError::InvalidInput);
    }
    let row = one(&backend.db,
        "SELECT public_document,EXISTS(SELECT 1 FROM content_withdrawals w WHERE (w.lesson_id,w.revision)=(r.lesson_id,r.revision)) AS withdrawn FROM lesson_revisions r WHERE lesson_id=$1 AND revision=$2",
        vec![id.into(),(revision as i32).into()]).await?.ok_or(AppError::NotFound)?;
    if field::<bool>(&row, "withdrawn")? {
        return Err(AppError::Gone);
    }
    let mut lesson: PublicLesson = serde_json::from_value(field(&row, "public_document")?)
        .map_err(|_| AppError::Unavailable)?;
    lesson.validate().map_err(|_| AppError::Unavailable)?;
    if lesson.id != id || lesson.revision != revision {
        return Err(AppError::Unavailable);
    }
    for asset in &mut lesson.media {
        let name = asset
            .url
            .strip_prefix("/api/media/")
            .ok_or(AppError::Unavailable)?;
        if name.contains('/') || name.contains('?') || name.contains('#') {
            return Err(AppError::Unavailable);
        }
        asset.url = format!("/api/v1/operator/lessons/{id}/revisions/{revision}/media/{name}");
    }
    Ok(lesson)
}
async fn lesson(
    auth: AuthSession,
    State(backend): State<Backend>,
    Path((id, revision)): Path<(String, u32)>,
) -> Result<Json<PublicLesson>, AppError> {
    require_operator(&auth)?;
    Ok(Json(read(&backend, &id, revision).await?))
}
async fn media(
    auth: AuthSession,
    axum::Extension(config): axum::Extension<PreviewMedia>,
    State(backend): State<Backend>,
    Path((id, revision, name)): Path<(String, u32, String)>,
) -> Result<axum::response::Response, AppError> {
    require_operator(&auth)?;
    let lesson = read(&backend, &id, revision).await?;
    let asset = lesson
        .media
        .into_iter()
        .find(|asset| asset.url.rsplit('/').next() == Some(name.as_str()))
        .ok_or(AppError::NotFound)?;
    crate::media::asset_response(config.root, asset, config.permits).await
}
