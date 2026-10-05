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
        self.validate_author().map_err(|_| AppError::InvalidInput)
    }
    /// Local author diagnostics; public callers retain the opaque AppError.
    pub fn validate_author(&self) -> anyhow::Result<()> {
        use anyhow::ensure;
        ensure!(identifier(&self.id), "/id: invalid release ID");
        ensure!(
            self.schema_version == "1.0",
            "/schemaVersion: unsupported schema version"
        );
        ensure!(
            self.levels.len() <= 20,
            "/levels: at most 20 levels are allowed"
        );
        let (mut levels, mut units, mut lessons) =
            (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
        for (li, level) in self.levels.iter().enumerate() {
            let level_path = format!("/levels/{li}");
            ensure!(
                identifier(&level.id) && levels.insert(&level.id),
                "{level_path}/id: invalid or duplicate level ID"
            );
            ensure!(
                text(&level.label),
                "{level_path}/label: expected nonempty label without control characters, at most 1000 bytes"
            );
            ensure!(
                !level.units.is_empty(),
                "{level_path}/units: level must contain a unit"
            );
            for (ui, unit) in level.units.iter().enumerate() {
                let unit_path = format!("{level_path}/units/{ui}");
                ensure!(
                    identifier(&unit.id) && units.insert(&unit.id),
                    "{unit_path}/id: invalid or duplicate unit ID"
                );
                ensure!(
                    text(&unit.title_zh),
                    "{unit_path}/titleZh: expected nonempty title without control characters, at most 1000 bytes"
                );
                ensure!(
                    !unit.lessons.is_empty(),
                    "{unit_path}/lessons: unit must contain a lesson"
                );
                for (ri, lesson) in unit.lessons.iter().enumerate() {
                    let lesson_path = format!("{unit_path}/lessons/{ri}");
                    ensure!(
                        identifier(&lesson.lesson_id) && lessons.insert(&lesson.lesson_id),
                        "{lesson_path}/lessonId: invalid or duplicate lesson reference"
                    );
                    ensure!(
                        lesson.revision > 0 && lesson.revision <= i32::MAX as u32,
                        "{lesson_path}/revision: expected positive database-compatible revision"
                    );
                    ensure!(
                        lessons.len() <= 5000,
                        "{unit_path}/lessons: at most 5000 lesson references are allowed"
                    );
                }
            }
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
    stage_impl(db, manifest, actor, reason, media_root)
        .await
        .map_err(|error| error.runtime)
}

/// Local CLI diagnostics use the same transaction and publication gates as stage.
pub async fn stage_author(
    db: &DatabaseConnection,
    manifest: &ReleaseManifest,
    actor: &str,
    reason: &str,
    media_root: &std::path::Path,
) -> anyhow::Result<()> {
    stage_impl(db, manifest, actor, reason, media_root)
        .await
        .map_err(|error| {
            anyhow::anyhow!(error.diagnostic.unwrap_or_else(|| {
                "/: staging database operation failed; verify release status before retrying".into()
            }))
        })
}

struct StageFailure {
    runtime: AppError,
    diagnostic: Option<String>,
}
impl From<AppError> for StageFailure {
    fn from(runtime: AppError) -> Self {
        Self {
            runtime,
            diagnostic: None,
        }
    }
}
impl StageFailure {
    fn at(runtime: AppError, pointer: &str, message: &str) -> Self {
        Self {
            runtime,
            diagnostic: Some(format!("{pointer}: {message}")),
        }
    }
}
async fn stage_impl(
    db: &DatabaseConnection,
    manifest: &ReleaseManifest,
    actor: &str,
    reason: &str,
    media_root: &std::path::Path,
) -> Result<(), StageFailure> {
    manifest.validate_author().map_err(|error| StageFailure {
        runtime: AppError::InvalidInput,
        diagnostic: Some(error.to_string()),
    })?;
    if !text(actor) || !text(reason) {
        return Err(StageFailure::at(
            AppError::InvalidInput,
            "/",
            "actor and reason must be nonempty, without control characters, at most 1000 bytes",
        ));
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
        return Err(StageFailure::at(
            AppError::Conflict,
            "/id",
            "release ID already exists; releases are immutable",
        ));
    }
    let mut entries = Vec::new();
    let mut source_hashes = Vec::new();
    for (li, level) in manifest.levels.iter().enumerate() {
        for (ui, unit) in level.units.iter().enumerate() {
            for (ri, entry) in unit.lessons.iter().enumerate() {
                let path = format!("/levels/{li}/units/{ui}/lessons/{ri}");
                let revision_path = format!("{path}/revision");
                let row=one(&tx,"SELECT public_document,server_document,EXISTS(SELECT 1 FROM content_withdrawals w WHERE (w.lesson_id,w.revision)=(r.lesson_id,r.revision)) AS withdrawn FROM lesson_revisions r WHERE lesson_id=$1 AND revision=$2 FOR SHARE",vec![entry.lesson_id.clone().into(),(entry.revision as i32).into()]).await?.ok_or_else(|| StageFailure::at(AppError::NotFound, &revision_path, "referenced lesson revision has not been imported"))?;
                if field::<bool>(&row, "withdrawn")? {
                    return Err(StageFailure::at(
                        AppError::Gone,
                        &revision_path,
                        "referenced lesson revision was withdrawn",
                    ));
                }
                let source: serde_json::Value = field(&row, "server_document")?;
                if !matches!(
                    crate::author_source::editorial(&source)
                        .map_err(|_| StageFailure::at(
                            AppError::InvalidInput,
                            &path,
                            "imported lesson has invalid editorial metadata"
                        ))?
                        .status,
                    crate::author_source::EditorialStatus::Reviewed
                ) {
                    return Err(StageFailure::at(
                        AppError::InvalidInput,
                        &path,
                        "imported lesson requires reviewed editorial status",
                    ));
                }
                let lesson = project_source(source.clone()).map_err(|error| {
                    StageFailure::at(
                        AppError::InvalidInput,
                        &path,
                        &format!("imported lesson validation failed: {error}"),
                    )
                })?;
                if lesson.level_id != level.id
                    || lesson.unit_id != unit.id
                    || lesson.id != entry.lesson_id
                    || lesson.revision != entry.revision
                    || serde_json::to_value(&lesson).map_err(|_| AppError::Unavailable)?
                        != field::<serde_json::Value>(&row, "public_document")?
                {
                    return Err(StageFailure::at(
                        AppError::InvalidInput,
                        &path,
                        "lesson level/unit/revision or stored public projection does not match this directory entry",
                    ));
                }
                Grader::from_author_source(&lesson, &source).map_err(|error| {
                    StageFailure::at(
                        AppError::InvalidInput,
                        &path,
                        &format!("imported lesson grading validation failed: {error}"),
                    )
                })?;
                crate::media::validate_lesson(&tx, &lesson, media_root).await.map_err(|error| StageFailure::at(error, &path, "registered visual/audio media failed publication validation; verify registration, rights and stored files"))?;
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
    tx.commit().await.map_err(|_| AppError::Unavailable)?;
    Ok(())
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

#[cfg(test)]
mod author_tests {
    use super::*;
    fn fixture() -> serde_json::Value {
        serde_json::from_str(include_str!("../../../docs/examples/catalog.release.json")).unwrap()
    }
    #[test]
    fn locates_release_fields_and_preserves_opaque_public_errors() {
        for (pointer, value) in [
            ("/id", serde_json::json!("bad release")),
            ("/schemaVersion", serde_json::json!("2.0")),
            ("/levels/0/id", serde_json::json!("bad level")),
            ("/levels/0/label", serde_json::json!(" ")),
            ("/levels/0/units", serde_json::json!([])),
            ("/levels/0/units/0/id", serde_json::json!("bad unit")),
            ("/levels/0/units/0/titleZh", serde_json::json!("\n")),
            ("/levels/0/units/0/lessons", serde_json::json!([])),
            (
                "/levels/0/units/0/lessons/0/lessonId",
                serde_json::json!("bad lesson"),
            ),
            ("/levels/0/units/0/lessons/0/revision", serde_json::json!(0)),
            (
                "/levels/0/units/0/lessons/0/revision",
                serde_json::json!(2147483648u32),
            ),
        ] {
            let mut source = fixture();
            *source.pointer_mut(pointer).unwrap() = value;
            let manifest: ReleaseManifest = serde_json::from_value(source).unwrap();
            assert!(
                manifest
                    .validate_author()
                    .unwrap_err()
                    .to_string()
                    .starts_with(&format!("{pointer}:"))
            );
            assert!(matches!(manifest.validate(), Err(AppError::InvalidInput)));
        }
        let manifest: ReleaseManifest = serde_json::from_value(fixture()).unwrap();
        assert!(manifest.validate().is_ok());
        let empty: ReleaseManifest = serde_json::from_value(
            serde_json::json!({"id":"empty","schemaVersion":"1.0","levels":[]}),
        )
        .unwrap();
        assert!(empty.validate().is_ok());
    }
    #[test]
    fn identifies_second_duplicate_and_bounds_catalog_size() {
        let mut source = fixture();
        let reference = source["levels"][0]["units"][0]["lessons"][0].clone();
        source["levels"][0]["units"][0]["lessons"]
            .as_array_mut()
            .unwrap()
            .push(reference);
        let manifest: ReleaseManifest = serde_json::from_value(source).unwrap();
        assert!(
            manifest
                .validate_author()
                .unwrap_err()
                .to_string()
                .starts_with("/levels/0/units/0/lessons/1/lessonId:")
        );
        let mut source = fixture();
        let unit = source["levels"][0]["units"][0].clone();
        source["levels"][0]["units"]
            .as_array_mut()
            .unwrap()
            .push(unit);
        let manifest: ReleaseManifest = serde_json::from_value(source).unwrap();
        assert!(
            manifest
                .validate_author()
                .unwrap_err()
                .to_string()
                .starts_with("/levels/0/units/1/id:")
        );
        let mut source = fixture();
        source["levels"][0]["units"][0]["lessons"] = serde_json::json!(
            (0..5001)
                .map(|i| serde_json::json!({"lessonId":format!("lesson-{i}"),"revision":1}))
                .collect::<Vec<_>>()
        );
        let manifest: ReleaseManifest = serde_json::from_value(source).unwrap();
        assert!(
            manifest
                .validate_author()
                .unwrap_err()
                .to_string()
                .starts_with("/levels/0/units/0/lessons:")
        );
        let mut source = fixture();
        source["levels"] = serde_json::json!(
            (0..21)
                .map(|i| serde_json::json!({"id":format!("level-{i}"),"label":"A1","units":[]}))
                .collect::<Vec<_>>()
        );
        let manifest: ReleaseManifest = serde_json::from_value(source).unwrap();
        assert!(
            manifest
                .validate_author()
                .unwrap_err()
                .to_string()
                .starts_with("/levels:")
        );
    }
}
