//! Shared immutable lesson import for local author tooling and authenticated operators.
use crate::{
    AppError,
    learning::{exec, field, one},
};
use sea_orm::{DatabaseConnection, TransactionTrait};
use serde_json::Value;
#[derive(Debug)]
pub struct RevisionConflict;
impl std::fmt::Display for RevisionConflict {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("/revision: lesson revision already exists; revisions are immutable")
    }
}
impl std::error::Error for RevisionConflict {}
pub async fn import(
    db: &DatabaseConnection,
    source: Value,
    actor: &str,
    reason: &str,
) -> anyhow::Result<brioche_course_contract::AdminImportResult> {
    import_impl(db, source, actor, reason, false).await
}
pub async fn import_retry(
    db: &DatabaseConnection,
    source: Value,
    actor: &str,
    reason: &str,
) -> anyhow::Result<brioche_course_contract::AdminImportResult> {
    import_impl(db, source, actor, reason, true).await
}
async fn import_impl(
    db: &DatabaseConnection,
    source: Value,
    actor: &str,
    reason: &str,
    allow_identical_retry: bool,
) -> anyhow::Result<brioche_course_contract::AdminImportResult> {
    anyhow::ensure!(
        !actor.trim().is_empty() && actor.len() <= 1000 && !actor.chars().any(char::is_control),
        "/: invalid import actor"
    );
    anyhow::ensure!(
        !reason.trim().is_empty() && reason.len() <= 1000 && !reason.chars().any(char::is_control),
        "/: invalid import reason"
    );
    crate::validate_source_schema(source.clone())?;
    crate::media::source_asset_refs(&source)?;
    crate::recording::source_audio_refs(&source)?;
    let source = crate::media::hydrate_source(db, source).await?;
    let source = crate::recording::hydrate_source(db, source).await?;
    let lesson = crate::project_source(source.clone())?;
    crate::grading::Grader::from_author_source(&lesson, &source)?;
    let tx = db.begin().await.map_err(|_| AppError::Unavailable)?;
    // Serialize import retries by their immutable identity, independent of the directory lock.
    exec(
        &tx,
        "SELECT pg_advisory_xact_lock(hashtextextended($1,0))",
        vec![format!("lesson-import:{}:{}", lesson.id, lesson.revision).into()],
    )
    .await?;
    let identity = vec![lesson.id.clone().into(), (lesson.revision as i32).into()];
    if let Some(existing) = one(
        &tx,
        "SELECT server_document FROM lesson_revisions WHERE lesson_id=$1 AND revision=$2",
        identity,
    )
    .await?
    {
        if !allow_identical_retry || field::<Value>(&existing, "server_document")? != source {
            return Err(RevisionConflict.into());
        }
    } else {
        exec(&tx,"INSERT INTO lesson_revisions(lesson_id,revision,published,public_document,server_document) VALUES($1,$2,false,$3,$4)",vec![lesson.id.clone().into(),(lesson.revision as i32).into(),serde_json::to_value(&lesson)?.into(),source.into()]).await?;
        exec(
            &tx,
            "INSERT INTO lesson_import_audit(lesson_id,revision,actor,reason) VALUES($1,$2,$3,$4)",
            vec![
                lesson.id.clone().into(),
                (lesson.revision as i32).into(),
                actor.into(),
                reason.into(),
            ],
        )
        .await?;
    }
    tx.commit().await.map_err(|_| AppError::Unavailable)?;
    Ok(brioche_course_contract::AdminImportResult {
        lesson_id: lesson.id,
        revision: lesson.revision,
    })
}
