//! Operator controls reuse content transactions; author snapshots remain immutable.
use crate::{
    AppError,
    identity::{AuthSession, Backend, require_operator},
    learning::{exec, field, one, owner},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use brioche_course_contract::{
    AdminActivateRequest, AdminLesson, AdminOverview, AdminRelease, AdminReviewRequest,
    AdminWithdrawRequest,
};
use sea_orm::{ConnectionTrait, DbBackend, IsolationLevel, Statement, TransactionTrait};
use serde_json::Value;

pub fn router(root: std::path::PathBuf) -> Router<Backend> {
    Router::new()
        .route("/api/v1/operator/overview", get(overview))
        .route(
            "/api/v1/operator/lessons/{id}/revisions/{revision}/review",
            post(review),
        )
        .route("/api/v1/operator/releases/activate", post(activate))
        .route(
            "/api/v1/operator/lessons/{id}/revisions/{revision}/withdraw",
            post(withdraw),
        )
        .layer(axum::Extension(root))
}
fn reason(value: &str) -> Result<(), AppError> {
    if value.trim().is_empty() || value.len() > 1000 || value.chars().any(char::is_control) {
        return Err(AppError::InvalidInput);
    }
    Ok(())
}
fn revision(id: &str, rev: u32) -> Result<(), AppError> {
    if !brioche_course_contract::valid_content_id(id)
        || !brioche_course_contract::valid_content_revision(rev)
    {
        return Err(AppError::InvalidInput);
    }
    Ok(())
}
fn generation(value: &str) -> Result<i64, AppError> {
    let result = value.parse::<i64>().map_err(|_| AppError::InvalidInput)?;
    if result < 0 || result.to_string() != value {
        return Err(AppError::InvalidInput);
    }
    Ok(result)
}
pub(crate) async fn approved<C: ConnectionTrait>(
    db: &C,
    id: &str,
    rev: u32,
    source: &Value,
) -> Result<bool, AppError> {
    let latest = one(db,"SELECT approved FROM editorial_reviews WHERE lesson_id=$1 AND revision=$2 ORDER BY version DESC LIMIT 1",vec![id.into(),(rev as i32).into()]).await?;
    match latest {
        Some(row) => field(&row, "approved"),
        None => Ok(matches!(
            crate::author_source::editorial(source)
                .map_err(|_| AppError::InvalidInput)?
                .status,
            crate::author_source::EditorialStatus::Reviewed
        )),
    }
}
async fn overview(
    auth: AuthSession,
    State(backend): State<Backend>,
) -> Result<Json<AdminOverview>, AppError> {
    require_operator(&auth)?;
    let tx = backend
        .db
        .begin_with_config(Some(IsolationLevel::RepeatableRead), None)
        .await
        .map_err(|_| AppError::Unavailable)?;
    let state = one(
        &tx,
        "SELECT active_release,generation FROM content_state WHERE singleton",
        vec![],
    )
    .await?
    .ok_or(AppError::Unavailable)?;
    let rows=tx.query_all_raw(Statement::from_string(DbBackend::Postgres,
        "SELECT r.lesson_id,r.revision,r.public_document,r.published,r.server_document->'editorial' AS editorial,v.version,v.approved,v.reason,EXISTS(SELECT 1 FROM content_withdrawals w WHERE (w.lesson_id,w.revision)=(r.lesson_id,r.revision)) AS withdrawn FROM lesson_revisions r LEFT JOIN LATERAL (SELECT version,approved,reason FROM editorial_reviews WHERE (lesson_id,revision)=(r.lesson_id,r.revision) ORDER BY version DESC LIMIT 1) v ON true ORDER BY r.lesson_id,r.revision DESC LIMIT 200")).await.map_err(|_|AppError::Unavailable)?;
    let mut lessons = Vec::new();
    for row in rows {
        let public: Value = field(&row, "public_document")?;
        let editorial: Value = field(&row, "editorial")?;
        lessons.push(AdminLesson {
            id: field(&row, "lesson_id")?,
            revision: field::<i32>(&row, "revision")? as u32,
            title: public["title"]["zh"]
                .as_str()
                .ok_or(AppError::Unavailable)?
                .into(),
            level: public["levelId"]
                .as_str()
                .ok_or(AppError::Unavailable)?
                .into(),
            unit: public["unitId"]
                .as_str()
                .ok_or(AppError::Unavailable)?
                .into(),
            published: field(&row, "published")?,
            withdrawn: field(&row, "withdrawn")?,
            approved: field::<Option<bool>>(&row, "approved")?
                .unwrap_or(editorial["status"] == "reviewed"),
            review_version: field::<Option<i32>>(&row, "version")?.unwrap_or(0) as u32,
            review_note: field::<Option<String>>(&row, "reason")?
                .unwrap_or_else(|| editorial["note"].as_str().unwrap_or("").into()),
        });
    }
    let rows=tx.query_all_raw(Statement::from_string(DbBackend::Postgres,"SELECT r.id,count(e.lesson_id) AS lesson_count FROM content_releases r LEFT JOIN release_entries e ON e.release_id=r.id GROUP BY r.id,r.created_at ORDER BY r.created_at DESC,r.id LIMIT 100")).await.map_err(|_|AppError::Unavailable)?;
    let releases = rows
        .iter()
        .map(|row| {
            Ok(AdminRelease {
                id: field(row, "id")?,
                lesson_count: u32::try_from(field::<i64>(row, "lesson_count")?)
                    .map_err(|_| AppError::Unavailable)?,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;
    let result = AdminOverview {
        generation: field::<i64>(&state, "generation")?.to_string(),
        active_release: field(&state, "active_release")?,
        lessons,
        releases,
    };
    tx.commit().await.map_err(|_| AppError::Unavailable)?;
    Ok(Json(result))
}
async fn review(
    auth: AuthSession,
    State(backend): State<Backend>,
    Path((id, rev)): Path<(String, u32)>,
    Json(request): Json<AdminReviewRequest>,
) -> Result<Json<AdminReviewRequest>, AppError> {
    require_operator(&auth)?;
    revision(&id, rev)?;
    reason(&request.reason)?;
    let actor = owner(&auth)?;
    let tx = backend
        .db
        .begin()
        .await
        .map_err(|_| AppError::Unavailable)?;
    one(
        &tx,
        "SELECT generation FROM content_state WHERE singleton FOR UPDATE",
        vec![],
    )
    .await?
    .ok_or(AppError::Unavailable)?;
    let row=one(&tx,"SELECT published,EXISTS(SELECT 1 FROM content_withdrawals WHERE lesson_id=$1 AND revision=$2) AS withdrawn FROM lesson_revisions WHERE lesson_id=$1 AND revision=$2 FOR UPDATE",vec![id.clone().into(),(rev as i32).into()]).await?.ok_or(AppError::NotFound)?;
    if field::<bool>(&row, "withdrawn")? {
        return Err(AppError::Gone);
    }
    // Published snapshots keep their approval; removal uses the explicit withdrawal operation.
    if field::<bool>(&row, "published")? {
        return Err(AppError::Conflict);
    }
    let latest=one(&tx,"SELECT version,approved,reason FROM editorial_reviews WHERE lesson_id=$1 AND revision=$2 ORDER BY version DESC LIMIT 1",vec![id.clone().into(),(rev as i32).into()]).await?;
    let current = latest
        .as_ref()
        .map(|row| field::<i32>(row, "version"))
        .transpose()?
        .unwrap_or(0) as u32;
    if current != request.version {
        // An identical retry of the immediately preceding decision is acknowledged without another row.
        if current == request.version.saturating_add(1)
            && latest.as_ref().is_some_and(|row| {
                field::<bool>(row, "approved").ok() == Some(request.approved)
                    && field::<String>(row, "reason").ok().as_deref() == Some(&request.reason)
            })
        {
            let original=one(&tx,"SELECT actor_id FROM editorial_reviews WHERE lesson_id=$1 AND revision=$2 AND version=$3",vec![id.into(),(rev as i32).into(),(current as i32).into()]).await?.ok_or(AppError::Unavailable)?;
            if field::<i64>(&original, "actor_id")? == actor {
                return Ok(Json(AdminReviewRequest {
                    version: current,
                    ..request
                }));
            }
        }
        return Err(AppError::Conflict);
    }
    let next = i32::try_from(current)
        .ok()
        .and_then(|v| v.checked_add(1))
        .ok_or(AppError::Unavailable)?;
    exec(&tx,"INSERT INTO editorial_reviews(lesson_id,revision,version,approved,actor_id,reason) VALUES($1,$2,$3,$4,$5,$6)",vec![id.into(),(rev as i32).into(),next.into(),request.approved.into(),actor.into(),request.reason.clone().into()]).await?;
    tx.commit().await.map_err(|_| AppError::Unavailable)?;
    Ok(Json(AdminReviewRequest {
        version: next as u32,
        ..request
    }))
}
async fn activate(
    auth: AuthSession,
    State(backend): State<Backend>,
    axum::Extension(root): axum::Extension<std::path::PathBuf>,
    Json(request): Json<AdminActivateRequest>,
) -> Result<Json<String>, AppError> {
    require_operator(&auth)?;
    reason(&request.reason)?;
    let actor = format!("user:{}", owner(&auth)?);
    let result = crate::content::activate(
        &backend.db,
        &request.release_id,
        generation(&request.generation)?,
        &actor,
        &request.reason,
        &root,
    )
    .await?;
    Ok(Json(result.to_string()))
}
async fn withdraw(
    auth: AuthSession,
    State(backend): State<Backend>,
    Path((id, rev)): Path<(String, u32)>,
    Json(request): Json<AdminWithdrawRequest>,
) -> Result<Json<String>, AppError> {
    require_operator(&auth)?;
    revision(&id, rev)?;
    reason(&request.reason)?;
    let actor = format!("user:{}", owner(&auth)?);
    let result = crate::content::withdraw(
        &backend.db,
        &id,
        rev,
        generation(&request.generation)?,
        &actor,
        &request.reason,
    )
    .await?;
    Ok(Json(result.to_string()))
}
