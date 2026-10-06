//! Operator controls reuse content transactions; author snapshots remain immutable.
use crate::{
    AppError,
    identity::{AuthSession, Backend, require_operator},
    learning::{exec, field, one, owner},
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use brioche_course_contract::{
    AdminActivateRequest, AdminDocumentRequest, AdminImportResult, AdminLesson, AdminOverview,
    AdminRelease, AdminReviewRequest, AdminWithdrawRequest,
};
use sea_orm::{ConnectionTrait, DbBackend, IsolationLevel, Statement, TransactionTrait};
use serde_json::Value;

pub fn router(root: std::path::PathBuf) -> Router<Backend> {
    Router::new()
        .route("/api/v1/operator/overview", get(overview))
        .route("/api/v1/operator/history", get(history))
        .route("/api/v1/operator/accounts", get(accounts))
        .route("/api/v1/operator/accounts/token", post(account_token))
        .route("/api/v1/operator/accounts/{id}/role", post(account_role))
        .route(
            "/api/v1/operator/lessons/{id}/revisions/{revision}/review",
            post(review),
        )
        .route("/api/v1/operator/releases/activate", post(activate))
        .route(
            "/api/v1/operator/releases/stage",
            post(stage).layer(axum::extract::DefaultBodyLimit::max(4 * 1024 * 1024)),
        )
        .route(
            "/api/v1/operator/lessons/import",
            post(import_lesson).layer(axum::extract::DefaultBodyLimit::max(4 * 1024 * 1024)),
        )
        .route(
            "/api/v1/operator/lessons/{id}/revisions/{revision}/withdraw",
            post(withdraw),
        )
        .layer(axum::Extension(root))
        .layer(axum::Extension(std::sync::Arc::new(
            tokio::sync::Semaphore::new(2),
        )))
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HistoryQuery {
    before_time: Option<String>,
    before_key: Option<String>,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AccountQuery {
    after_id: Option<String>,
    q: Option<String>,
}
async fn accounts(
    auth: AuthSession,
    State(backend): State<Backend>,
    Query(query): Query<AccountQuery>,
) -> Result<Json<brioche_course_contract::AdminAccounts>, AppError> {
    use brioche_course_contract::{AdminAccount, AdminAccounts};
    require_operator(&auth)?;
    let after = generation(query.after_id.as_deref().unwrap_or("0"))?;
    let search = query.q.unwrap_or_default().trim().to_owned();
    if search.len() > 300 || search.chars().count() > 100 || search.chars().any(char::is_control) {
        return Err(AppError::InvalidInput);
    }
    let rows=backend.db.query_all_raw(Statement::from_sql_and_values(DbBackend::Postgres,"SELECT id,email,display_name,role FROM users WHERE id>$1 AND ($2='' OR strpos(lower(email||' '||display_name),lower($2))>0) ORDER BY id LIMIT 21",vec![after.into(),search.into()])).await.map_err(|_|AppError::Unavailable)?;
    let has_more = rows.len() > 20;
    let mut items = Vec::new();
    for row in rows.into_iter().take(20) {
        items.push(AdminAccount {
            id: field::<i64>(&row, "id")?.to_string(),
            email: field(&row, "email")?,
            display_name: field(&row, "display_name")?,
            role: field(&row, "role")?,
        });
    }
    let next_id = if has_more {
        items.last().map(|item| item.id.clone())
    } else {
        None
    };
    Ok(Json(AdminAccounts { items, next_id }))
}
async fn account_token(
    auth: AuthSession,
    State(backend): State<Backend>,
    Json(request): Json<brioche_course_contract::AdminTokenRequest>,
) -> Result<Json<brioche_course_contract::AdminTokenResult>, AppError> {
    use brioche_course_contract::{AdminTokenKind, AdminTokenResult};
    require_operator(&auth)?;
    let reset = matches!(request.kind, AdminTokenKind::Reset);
    if reset && request.operator {
        return Err(AppError::InvalidInput);
    }
    let email = crate::identity::normalize_email(&request.email)?;
    let token = backend
        .issue_operator_token(
            &email,
            reset,
            request.operator,
            owner(&auth)?,
            &request.reason,
        )
        .await?;
    Ok(Json(AdminTokenResult {
        token,
        email,
        kind: request.kind,
        expires_in_seconds: if reset { 1800 } else { 172800 },
    }))
}
async fn account_role(
    auth: AuthSession,
    State(backend): State<Backend>,
    Path(id): Path<String>,
    Json(request): Json<brioche_course_contract::AdminRoleRequest>,
) -> Result<Json<brioche_course_contract::AdminAccount>, AppError> {
    use brioche_course_contract::{AdminAccount, AdminAccountRole};
    require_operator(&auth)?;
    reason(&request.reason)?;
    let target = generation(&id)?;
    if target == 0 {
        return Err(AppError::InvalidInput);
    }
    let role_name = |role| match role {
        AdminAccountRole::Learner => "learner",
        AdminAccountRole::Operator => "operator",
    };
    let expected = role_name(request.expected_role);
    let desired = role_name(request.role);
    let actor = owner(&auth)?;
    let tx = backend
        .db
        .begin()
        .await
        .map_err(|_| AppError::Unavailable)?;
    exec(
        &tx,
        "SELECT pg_advisory_xact_lock(hashtextextended('account-admin',0))",
        vec![],
    )
    .await?;
    let operator = one(
        &tx,
        "SELECT role FROM users WHERE id=$1",
        vec![actor.into()],
    )
    .await?
    .ok_or(AppError::Forbidden)?;
    if field::<String>(&operator, "role")? != "operator" {
        return Err(AppError::Forbidden);
    }
    let row = one(
        &tx,
        "SELECT email,display_name,role FROM users WHERE id=$1 FOR UPDATE",
        vec![target.into()],
    )
    .await?
    .ok_or(AppError::NotFound)?;
    let current: String = field(&row, "role")?;
    if current != expected {
        return Err(AppError::Conflict);
    }
    let email: String = field(&row, "email")?;
    if current != desired {
        if current == "operator" {
            let count = one(
                &tx,
                "SELECT count(*) AS n FROM users WHERE role='operator'",
                vec![],
            )
            .await?
            .ok_or(AppError::Unavailable)?;
            if field::<i64>(&count, "n")? <= 1 {
                return Err(AppError::Conflict);
            }
        }
        exec(
            &tx,
            "UPDATE users SET role=$1 WHERE id=$2",
            vec![desired.into(), target.into()],
        )
        .await?;
        exec(&tx, "INSERT INTO account_admin_audit(action,actor_id,target_email,reason,details) VALUES('role',$1,$2,$3,$4)", vec![actor.into(),email.clone().into(),request.reason.into(),serde_json::json!({"userId":id,"from":current,"to":desired}).into()]).await?;
    }
    let account = AdminAccount {
        id,
        email,
        display_name: field(&row, "display_name")?,
        role: desired.into(),
    };
    tx.commit().await.map_err(|_| AppError::Unavailable)?;
    Ok(Json(account))
}
async fn history(
    auth: AuthSession,
    State(backend): State<Backend>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<brioche_course_contract::AdminHistory>, AppError> {
    use brioche_course_contract::{AdminHistory, AdminHistoryCursor, AdminHistoryItem};
    require_operator(&auth)?;
    if query.before_time.is_some() != query.before_key.is_some() {
        return Err(AppError::InvalidInput);
    }
    if let Some(time) = &query.before_time
        && (time.len() > 40 || time.parse::<jiff::Timestamp>().is_err())
    {
        return Err(AppError::InvalidInput);
    }
    if let Some(key) = &query.before_key
        && (key.is_empty()
            || key.len() > 256
            || !key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_:".contains(&b)))
    {
        return Err(AppError::InvalidInput);
    }
    let rows = backend.db.query_all_raw(Statement::from_sql_and_values(DbBackend::Postgres, r#"
        WITH events AS (
            SELECT 'review:'||lesson_id||':'||revision||':'||version AS key,
                CASE WHEN approved THEN 'approve' ELSE 'reject' END AS action,
                lesson_id||' v'||revision AS target, 'user:'||actor_id AS actor, reason, created_at
            FROM editorial_reviews
            UNION ALL
            SELECT 'content:'||id, action, COALESCE(release_id,lesson_id||' v'||revision,'未指定对象'), actor, reason, created_at FROM content_audit
            UNION ALL
            SELECT 'import:'||lesson_id||':'||revision, 'import', lesson_id||' v'||revision, actor, reason, created_at FROM lesson_import_audit
            UNION ALL
            SELECT 'account:'||id, CASE WHEN action='invite' AND details->>'role'='operator' THEN 'inviteOperator' ELSE action END, target_email, 'user:'||actor_id, reason, created_at FROM account_admin_audit
        )
        SELECT key,action,target,actor,reason,to_char(created_at AT TIME ZONE 'UTC','YYYY-MM-DD"T"HH24:MI:SS.US"Z"') AS created_at
        FROM events WHERE $1::timestamptz IS NULL OR (created_at,key COLLATE "C") < ($1::timestamptz,$2::text COLLATE "C")
        ORDER BY created_at DESC,key COLLATE "C" DESC LIMIT 21
    "#, vec![query.before_time.into(), query.before_key.into()])).await.map_err(|_|AppError::Unavailable)?;
    let has_more = rows.len() > 20;
    let mut items = Vec::new();
    for row in rows.into_iter().take(20) {
        items.push(AdminHistoryItem {
            key: field(&row, "key")?,
            action: field(&row, "action")?,
            target: field(&row, "target")?,
            actor: field(&row, "actor")?,
            reason: field(&row, "reason")?,
            created_at: field(&row, "created_at")?,
        });
    }
    let next = if has_more {
        items.last().map(|item| AdminHistoryCursor {
            before_time: item.created_at.clone(),
            before_key: item.key.clone(),
        })
    } else {
        None
    };
    Ok(Json(AdminHistory { items, next }))
}
async fn import_lesson(
    auth: AuthSession,
    State(backend): State<Backend>,
    axum::Extension(permits): axum::Extension<std::sync::Arc<tokio::sync::Semaphore>>,
    Json(request): Json<AdminDocumentRequest>,
) -> Result<Json<AdminImportResult>, AppError> {
    require_operator(&auth)?;
    reason(&request.reason)?;
    let _permit = permits.try_acquire().map_err(|_| AppError::RateLimited)?;
    let source = tokio::task::spawn_blocking(move || {
        let source = crate::author_json::parse_document(request.document.as_bytes())?;
        crate::validate_source_schema(source.clone())?;
        Ok::<_, anyhow::Error>(source)
    })
    .await
    .map_err(|_| AppError::Unavailable)?
    .map_err(|_| AppError::InvalidInput)?;
    let actor = format!("user:{}", owner(&auth)?);
    let imported = crate::author_import::import_retry(&backend.db, source, &actor, &request.reason)
        .await
        .map_err(|error| {
            if error.is::<crate::author_import::RevisionConflict>() {
                return AppError::Conflict;
            }
            error
                .downcast_ref::<AppError>()
                .map_or(AppError::InvalidInput, |_| AppError::Unavailable)
        })?;
    Ok(Json(imported))
}
async fn stage(
    auth: AuthSession,
    State(backend): State<Backend>,
    axum::Extension(root): axum::Extension<std::path::PathBuf>,
    axum::Extension(permits): axum::Extension<std::sync::Arc<tokio::sync::Semaphore>>,
    Json(request): Json<AdminDocumentRequest>,
) -> Result<Json<String>, AppError> {
    require_operator(&auth)?;
    reason(&request.reason)?;
    let _permit = permits.try_acquire().map_err(|_| AppError::RateLimited)?;
    let manifest: crate::content::ReleaseManifest = tokio::task::spawn_blocking(move || {
        let value = crate::author_json::parse_document(request.document.as_bytes())?;
        let manifest: crate::content::ReleaseManifest = crate::author_json::from_value(value, "")?;
        manifest.validate_author()?;
        Ok::<_, anyhow::Error>(manifest)
    })
    .await
    .map_err(|_| AppError::Unavailable)?
    .map_err(|_| AppError::InvalidInput)?;
    let actor = format!("user:{}", owner(&auth)?);
    crate::content::stage(&backend.db, &manifest, &actor, &request.reason, &root).await?;
    Ok(Json(manifest.id))
}
pub(crate) fn reason(value: &str) -> Result<(), AppError> {
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
