//! Immutable directory releases; operations are local operator CLI transactions.
use crate::{
    AppError,
    grading::Grader,
    learning::{exec, field, hash, one},
    project_source,
};
use brioche_course_contract::{Catalog, Level, PublicLesson, Unit};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

fn search_text(value: &str) -> String {
    value
        .nfkd()
        .filter(|c| !is_combining_mark(*c))
        .collect::<String>()
        .to_lowercase()
}
pub fn search_terms(query: &str) -> Result<Vec<String>, AppError> {
    if query.chars().count() > 120 || query.chars().any(|c| c.is_control() && !c.is_whitespace()) {
        return Err(AppError::InvalidInput);
    }
    Ok(search_text(query)
        .split_whitespace()
        .map(str::to_owned)
        .collect())
}
pub fn search_catalog(mut catalog: Catalog, terms: &[String]) -> Catalog {
    if terms.is_empty() {
        return catalog;
    }
    for level in &mut catalog.levels {
        for unit in &mut level.units {
            unit.lessons.retain(|lesson| {
                let text = search_text(&format!(
                    "{} {} {} {} {}",
                    level.label, unit.title_zh, lesson.title.zh, lesson.title.fr, lesson.summary_zh
                ));
                terms.iter().all(|term| text.contains(term))
            });
        }
        level.units.retain(|unit| !unit.lessons.is_empty());
    }
    catalog.levels.retain(|level| !level.units.is_empty());
    catalog
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReleaseManifest {
    pub id: String,
    pub schema_version: String,
    pub levels: Vec<ReleaseLevel>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReleaseLevel {
    pub id: String,
    pub label: String,
    pub units: Vec<ReleaseUnit>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReleaseUnit {
    pub id: String,
    pub title_zh: String,
    pub lessons: Vec<RevisionRef>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RevisionRef {
    pub lesson_id: String,
    pub revision: u32,
}
fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 100
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
fn text(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 1000 && !value.chars().any(char::is_control)
}
impl ReleaseManifest {
    pub fn deserialize_file(path: &str) -> anyhow::Result<Self> {
        crate::author_json::load(path)
    }
    pub fn validate(&self) -> Result<(), AppError> {
        if !identifier(&self.id) || self.schema_version != "1.0" || self.levels.len() > 20 {
            return Err(AppError::InvalidInput);
        }
        let (mut levels, mut units, mut lessons) =
            (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
        for level in &self.levels {
            if !identifier(&level.id)
                || !text(&level.label)
                || !levels.insert(&level.id)
                || level.units.is_empty()
            {
                return Err(AppError::InvalidInput);
            }
            for unit in &level.units {
                if !identifier(&unit.id)
                    || !text(&unit.title_zh)
                    || !units.insert(&unit.id)
                    || unit.lessons.is_empty()
                {
                    return Err(AppError::InvalidInput);
                }
                for lesson in &unit.lessons {
                    if !identifier(&lesson.lesson_id)
                        || lesson.revision == 0
                        || lesson.revision > i32::MAX as u32
                        || !lessons.insert(&lesson.lesson_id)
                    {
                        return Err(AppError::InvalidInput);
                    }
                }
            }
        }
        if lessons.len() > 5000 {
            return Err(AppError::InvalidInput);
        }
        Ok(())
    }
}
pub async fn stage(
    db: &DatabaseConnection,
    manifest: &ReleaseManifest,
    actor: &str,
    reason: &str,
    media_root: &std::path::Path,
) -> Result<(), AppError> {
    manifest.validate()?;
    if !text(actor) || !text(reason) {
        return Err(AppError::InvalidInput);
    }
    let tx = db.begin().await.map_err(|_| AppError::Unavailable)?;
    // All content mutations lock the singleton before revision rows: no activation/withdrawal deadlock.
    let state = one(
        &tx,
        "SELECT generation FROM content_state WHERE singleton FOR UPDATE",
        vec![],
    )
    .await?
    .ok_or(AppError::Unavailable)?;
    if one(
        &tx,
        "SELECT id FROM content_releases WHERE id=$1",
        vec![manifest.id.clone().into()],
    )
    .await?
    .is_some()
    {
        return Err(AppError::Conflict);
    }
    let mut entries = Vec::new();
    let mut source_hashes = Vec::new();
    for level in &manifest.levels {
        for unit in &level.units {
            for entry in &unit.lessons {
                let row=one(&tx,"SELECT public_document,server_document,EXISTS(SELECT 1 FROM content_withdrawals w WHERE (w.lesson_id,w.revision)=(r.lesson_id,r.revision)) AS withdrawn FROM lesson_revisions r WHERE lesson_id=$1 AND revision=$2 FOR SHARE",vec![entry.lesson_id.clone().into(),(entry.revision as i32).into()]).await?.ok_or(AppError::NotFound)?;
                if field::<bool>(&row, "withdrawn")? {
                    return Err(AppError::Gone);
                }
                let source: serde_json::Value = field(&row, "server_document")?;
                if !matches!(
                    crate::author_source::editorial(&source)
                        .map_err(|_| AppError::InvalidInput)?
                        .status,
                    crate::author_source::EditorialStatus::Reviewed
                ) {
                    return Err(AppError::InvalidInput);
                }
                let lesson = project_source(source.clone()).map_err(|_| AppError::InvalidInput)?;
                if lesson.level_id != level.id
                    || lesson.unit_id != unit.id
                    || lesson.id != entry.lesson_id
                    || lesson.revision != entry.revision
                    || serde_json::to_value(&lesson).map_err(|_| AppError::Unavailable)?
                        != field::<serde_json::Value>(&row, "public_document")?
                {
                    return Err(AppError::InvalidInput);
                }
                Grader::from_source(&lesson, &source).map_err(|_| AppError::InvalidInput)?;
                crate::media::validate_lesson(&tx, &lesson, media_root).await?;
                source_hashes.push(hash(&source)?);
                entries.push(entry);
            }
        }
    }
    exec(
        &tx,
        "INSERT INTO content_releases(id,manifest,content_hash) VALUES($1,$2,$3)",
        vec![
            manifest.id.clone().into(),
            serde_json::to_value(manifest)
                .map_err(|_| AppError::Unavailable)?
                .into(),
            hash(&serde_json::json!({"manifest":manifest,"sources":source_hashes}))?.into(),
        ],
    )
    .await?;
    for (position, entry) in entries.into_iter().enumerate() {
        exec(&tx,"INSERT INTO release_entries(release_id,lesson_id,revision,position) VALUES($1,$2,$3,$4)",vec![manifest.id.clone().into(),entry.lesson_id.clone().into(),(entry.revision as i32).into(),(position as i32).into()]).await?;
    }
    exec(&tx,"INSERT INTO content_audit(action,actor,reason,release_id,generation) VALUES('stage',$1,$2,$3,$4)",vec![actor.into(),reason.into(),manifest.id.clone().into(),field::<i64>(&state,"generation")?.into()]).await?;
    tx.commit().await.map_err(|_| AppError::Unavailable)
}
pub async fn activate(
    db: &DatabaseConnection,
    id: &str,
    expected: i64,
    actor: &str,
    reason: &str,
    media_root: &std::path::Path,
) -> Result<i64, AppError> {
    if !identifier(id) || expected < 0 || !text(actor) || !text(reason) {
        return Err(AppError::InvalidInput);
    }
    let tx = db.begin().await.map_err(|_| AppError::Unavailable)?;
    let state = one(
        &tx,
        "SELECT generation FROM content_state WHERE singleton FOR UPDATE",
        vec![],
    )
    .await?
    .ok_or(AppError::Unavailable)?;
    let generation = field::<i64>(&state, "generation")?;
    if expected != generation {
        return Err(AppError::Conflict);
    }
    if one(
        &tx,
        "SELECT id FROM content_releases WHERE id=$1",
        vec![id.into()],
    )
    .await?
    .is_none()
    {
        return Err(AppError::NotFound);
    }
    if one(&tx,"SELECT 1 AS n FROM release_entries e JOIN content_withdrawals w USING(lesson_id,revision) WHERE e.release_id=$1 LIMIT 1",vec![id.into()]).await?.is_some(){return Err(AppError::Gone);}
    let next = generation.checked_add(1).ok_or(AppError::Unavailable)?;
    let rows=tx.query_all_raw(Statement::from_sql_and_values(DbBackend::Postgres,"SELECT r.public_document FROM release_entries e JOIN lesson_revisions r USING(lesson_id,revision) WHERE e.release_id=$1 ORDER BY e.position",[id.into()])).await.map_err(|_|AppError::Unavailable)?;
    for row in rows {
        let lesson: PublicLesson = serde_json::from_value(field(&row, "public_document")?)
            .map_err(|_| AppError::Unavailable)?;
        crate::media::validate_lesson(&tx, &lesson, media_root).await?;
    }
    exec(&tx,"UPDATE lesson_revisions r SET published=true FROM release_entries e WHERE e.release_id=$1 AND (r.lesson_id,r.revision)=(e.lesson_id,e.revision)",vec![id.into()]).await?;
    exec(
        &tx,
        "UPDATE content_state SET active_release=$1,generation=$2 WHERE singleton",
        vec![id.into(), next.into()],
    )
    .await?;
    exec(&tx,"INSERT INTO content_audit(action,actor,reason,release_id,generation) VALUES('activate',$1,$2,$3,$4)",vec![actor.into(),reason.into(),id.into(),next.into()]).await?;
    tx.commit().await.map_err(|_| AppError::Unavailable)?;
    Ok(next)
}
pub async fn withdraw(
    db: &DatabaseConnection,
    id: &str,
    revision: u32,
    expected: i64,
    actor: &str,
    reason: &str,
) -> Result<i64, AppError> {
    if !identifier(id)
        || revision == 0
        || revision > i32::MAX as u32
        || expected < 0
        || !text(actor)
        || !text(reason)
    {
        return Err(AppError::InvalidInput);
    }
    let tx = db.begin().await.map_err(|_| AppError::Unavailable)?;
    let state = one(
        &tx,
        "SELECT generation FROM content_state WHERE singleton FOR UPDATE",
        vec![],
    )
    .await?
    .ok_or(AppError::Unavailable)?;
    if field::<i64>(&state, "generation")? != expected {
        return Err(AppError::Conflict);
    }
    if exec(
        &tx,
        "UPDATE lesson_revisions SET published=false WHERE lesson_id=$1 AND revision=$2",
        vec![id.into(), (revision as i32).into()],
    )
    .await?
        != 1
    {
        return Err(AppError::NotFound);
    }
    if exec(
        &tx,
        "INSERT INTO content_withdrawals(lesson_id,revision) VALUES($1,$2) ON CONFLICT DO NOTHING",
        vec![id.into(), (revision as i32).into()],
    )
    .await?
        != 1
    {
        return Err(AppError::Gone);
    }
    let next = expected.checked_add(1).ok_or(AppError::Unavailable)?;
    exec(
        &tx,
        "UPDATE content_state SET generation=$1 WHERE singleton",
        vec![next.into()],
    )
    .await?;
    exec(&tx,"INSERT INTO content_audit(action,actor,reason,lesson_id,revision,generation) VALUES('withdraw',$1,$2,$3,$4,$5)",vec![actor.into(),reason.into(),id.into(),(revision as i32).into(),next.into()]).await?;
    tx.commit().await.map_err(|_| AppError::Unavailable)?;
    Ok(next)
}
pub async fn catalog<C: ConnectionTrait>(db: &C) -> Result<Catalog, AppError> {
    // One SQL statement observes the pointer, immutable manifest and available revisions together.
    let rows=db.query_all_raw(Statement::from_string(DbBackend::Postgres,"SELECT cr.manifest,r.public_document FROM content_state s JOIN content_releases cr ON cr.id=s.active_release LEFT JOIN release_entries e ON e.release_id=cr.id LEFT JOIN lesson_revisions r ON (r.lesson_id,r.revision)=(e.lesson_id,e.revision) AND r.published WHERE s.singleton ORDER BY e.position")).await.map_err(|_|AppError::Unavailable)?;
    let Some(first) = rows.first() else {
        return Ok(Catalog {
            levels: vec![],
            development_fixture: false,
        });
    };
    let manifest: ReleaseManifest =
        serde_json::from_value(field(first, "manifest")?).map_err(|_| AppError::Unavailable)?;
    let mut lessons = BTreeMap::new();
    for row in &rows {
        if let Some(document) = field::<Option<serde_json::Value>>(row, "public_document")? {
            let lesson: PublicLesson =
                serde_json::from_value(document).map_err(|_| AppError::Unavailable)?;
            lesson.validate().map_err(|_| AppError::Unavailable)?;
            lessons.insert(lesson.id.clone(), lesson.summary());
        }
    }
    Ok(Catalog {
        development_fixture: false,
        levels: manifest
            .levels
            .into_iter()
            .filter_map(|level| {
                let units: Vec<_> = level
                    .units
                    .into_iter()
                    .filter_map(|unit| {
                        let entries: Vec<_> = unit
                            .lessons
                            .into_iter()
                            .filter_map(|entry| lessons.remove(&entry.lesson_id))
                            .collect();
                        (!entries.is_empty()).then_some(Unit {
                            id: unit.id,
                            title_zh: unit.title_zh,
                            lessons: entries,
                        })
                    })
                    .collect();
                (!units.is_empty()).then_some(Level {
                    id: level.id,
                    label: level.label,
                    units,
                })
            })
            .collect(),
    })
}
