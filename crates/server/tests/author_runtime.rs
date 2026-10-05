//! Real author commands against an explicitly supplied, disposable PostgreSQL schema.
use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};
use sea_orm_migration::MigratorTrait;
use serde_json::{Value, json};
use std::{
    path::Path,
    process::{Command, Output},
};
#[path = "support/assets.rs"]
mod asset_fixtures;

fn invoke(url: &str, root: &Path, command: &str, file: &Path) -> Output {
    let mut process = Command::new(env!("CARGO_BIN_EXE_brioche-server"));
    process.args([command, file.to_str().unwrap()]);
    if command == "release-stage" {
        process.args(["protocol-test", "isolated synthetic author test"]);
    }
    process
        .env("DATABASE_URL", url)
        .env("CONTENT_MODE", "database")
        .env("APP_ENV", "production")
        .env("MEDIA_ROOT", root)
        .output()
        .unwrap()
}
fn write(file: &Path, value: &Value) -> String {
    let text = serde_json::to_string_pretty(value)
        .unwrap()
        .replace('\n', "\r\n");
    std::fs::write(file, &text).unwrap();
    text
}
fn located(output: Output, file: &Path, text: &str, pointer: &str, offset: usize, reason: &str) {
    let error = String::from_utf8_lossy(&output.stderr);
    let before = &text[..offset];
    let line = before.bytes().filter(|b| *b == b'\n').count() + 1;
    let column = before.rsplit('\n').next().unwrap().chars().count() + 1;
    assert!(!output.status.success());
    assert!(
        error.contains(&format!("{}:{line}:{column}: {pointer}:", file.display())),
        "{error}"
    );
    assert!(error.contains(reason), "{error}");
    assert!(
        !error.contains("INSERT INTO"),
        "database details must stay private: {error}"
    );
}
async fn count(db: &sea_orm::DatabaseConnection, table: &str) -> i64 {
    db.query_one_raw(Statement::from_string(
        DbBackend::Postgres,
        format!("SELECT count(*) AS n FROM {table}"),
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get("", "n")
    .unwrap()
}

#[tokio::test]
#[ignore = "set TEST_DATABASE_URL to a dedicated PostgreSQL database"]
async fn import_and_stage_cli_locate_original_source_and_preserve_atomicity() {
    let base = std::env::var("TEST_DATABASE_URL").expect("dedicated test database required");
    let admin = Database::connect(&base).await.unwrap();
    let schema = format!(
        "author_runtime_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    admin
        .execute_unprepared(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let mut url = url::Url::parse(&base).unwrap();
    url.query_pairs_mut()
        .append_pair("options", &format!("-c search_path={schema}"));
    let db = Database::connect(url.as_str()).await.unwrap();
    brioche_migration::Migrator::up(&db, None).await.unwrap();
    let root = asset_fixtures::fixture_assets(&db, &schema).await;
    let lesson_file = root.join("lesson.json");
    let release_file = root.join("release.json");
    let mut source = brioche_server::development_source().unwrap();
    source["assetRefs"] = asset_fixtures::fixture_refs();
    for (field, pointer, marker, reason) in [
        (
            "assetRefs",
            "/assetRefs/0/revision",
            7991,
            "registered asset revision missing",
        ),
        (
            "audioRefs",
            "/audioRefs/0/revision",
            7992,
            "registered recording revision missing",
        ),
    ] {
        let mut invalid = source.clone();
        invalid[field] = json!([{"assetId":"missing-registration", "revision":marker}]);
        let text = write(&lesson_file, &invalid);
        located(
            invoke(url.as_str(), &root, "import", &lesson_file),
            &lesson_file,
            &text,
            pointer,
            text.find(&marker.to_string()).unwrap(),
            reason,
        );
        assert_eq!(count(&db, "lesson_revisions").await, 0);
    }
    let mut invalid = source.clone();
    invalid["serverOnly"]["grading"]["exercise-intention"]["correctOptionId"] =
        json!("private-invalid-option");
    let text = write(&lesson_file, &invalid);
    let output = invoke(url.as_str(), &root, "import", &lesson_file);
    assert!(!String::from_utf8_lossy(&output.stderr).contains("private-invalid-option"));
    located(
        output,
        &lesson_file,
        &text,
        "/serverOnly/grading/exercise-intention/correctOptionId",
        text.find("\"private-invalid-option\"").unwrap(),
        "unknown option reference",
    );
    assert_eq!(count(&db, "lesson_revisions").await, 0);
    invalid = source.clone();
    invalid["revision"] = json!(2147483648u64);
    let text = write(&lesson_file, &invalid);
    located(
        invoke(url.as_str(), &root, "import", &lesson_file),
        &lesson_file,
        &text,
        "/revision",
        text.find("2147483648").unwrap(),
        "revision exceeds database range",
    );
    assert_eq!(count(&db, "lesson_revisions").await, 0);

    let text = write(&lesson_file, &source);
    let output = invoke(url.as_str(), &root, "import", &lesson_file);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    located(
        invoke(url.as_str(), &root, "import", &lesson_file),
        &lesson_file,
        &text,
        "/revision",
        text.rfind("\"revision\": 1").unwrap() + "\"revision\": ".len(),
        "already exists",
    );
    assert_eq!(count(&db, "lesson_revisions").await, 1);
    // A second synthetic revision is reviewed only within this disposable protocol test.
    let mut reviewed = source.clone();
    reviewed["id"] = json!("author-reviewed");
    reviewed["editorial"]["status"] = json!("reviewed");
    write(&lesson_file, &reviewed);
    assert!(
        invoke(url.as_str(), &root, "import", &lesson_file)
            .status
            .success()
    );
    let manifest = json!({"id":"author-release", "schemaVersion":"1.0", "levels":[{
    "id":source["levelId"],"label":"A1 入门","units":[{
        "id":source["unitId"],"titleZh":"面包店","lessons":[
            {"lessonId":"author-reviewed","revision":1},
            {"lessonId":source["id"],"revision":1}
        ]}]}]});
    let mut missing = manifest.clone();
    missing["levels"][0]["units"][0]["lessons"][1]["revision"] = json!(7993);
    let text = write(&release_file, &missing);
    located(
        invoke(url.as_str(), &root, "release-stage", &release_file),
        &release_file,
        &text,
        "/levels/0/units/0/lessons/1/revision",
        text.find("7993").unwrap(),
        "has not been imported",
    );
    let text = write(&release_file, &manifest);
    let lesson_marker = text
        .find(&format!("\"lessonId\": {}", source["id"]))
        .unwrap();
    let entry_offset = text[..lesson_marker].rfind('{').unwrap();
    located(
        invoke(url.as_str(), &root, "release-stage", &release_file),
        &release_file,
        &text,
        "/levels/0/units/0/lessons/1",
        entry_offset,
        "requires reviewed editorial status",
    );
    assert_eq!(count(&db, "content_releases").await, 0);
    assert_eq!(count(&db, "release_entries").await, 0);
    assert_eq!(count(&db, "content_audit").await, 0);
    let mut valid = manifest.clone();
    valid["levels"][0]["units"][0]["lessons"]
        .as_array_mut()
        .unwrap()
        .pop();
    let mut mismatch = valid.clone();
    mismatch["levels"][0]["units"][0]["id"] = json!("different-unit");
    let text = write(&release_file, &mismatch);
    let marker = text.find("\"lessonId\": \"author-reviewed\"").unwrap();
    let offset = text[..marker].rfind('{').unwrap();
    located(
        invoke(url.as_str(), &root, "release-stage", &release_file),
        &release_file,
        &text,
        "/levels/0/units/0/lessons/0",
        offset,
        "does not match",
    );
    let text = write(&release_file, &valid);
    let descriptor = db.query_one_raw(Statement::from_string(DbBackend::Postgres,
        "SELECT descriptor FROM media_assets WHERE asset_id='art-bakery-morning' AND revision=1"))
        .await.unwrap().unwrap().try_get::<Value>("", "descriptor").unwrap();
    let stored = root.join(format!("{}.svg", descriptor["sha256"].as_str().unwrap()));
    let original_bytes = std::fs::read(&stored).unwrap();
    std::fs::write(&stored, b"corrupt protocol fixture").unwrap();
    let marker = text.find("\"lessonId\": \"author-reviewed\"").unwrap();
    let offset = text[..marker].rfind('{').unwrap();
    let rejected = invoke(url.as_str(), &root, "release-stage", &release_file);
    std::fs::write(&stored, original_bytes).unwrap();
    located(
        rejected,
        &release_file,
        &text,
        "/levels/0/units/0/lessons/0",
        offset,
        "media failed publication validation",
    );
    assert_eq!(count(&db, "content_releases").await, 0);
    assert_eq!(count(&db, "content_audit").await, 0);
    let output = invoke(url.as_str(), &root, "release-stage", &release_file);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(count(&db, "content_releases").await, 1);
    assert_eq!(count(&db, "release_entries").await, 1);
    assert_eq!(count(&db, "content_audit").await, 1);
    located(
        invoke(url.as_str(), &root, "release-stage", &release_file),
        &release_file,
        &text,
        "/id",
        text.find("\"author-release\"").unwrap(),
        "already exists",
    );
    let state = db
        .query_one_raw(Statement::from_string(
            DbBackend::Postgres,
            "SELECT active_release,generation FROM content_state WHERE singleton",
        ))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        state
            .try_get::<Option<String>>("", "active_release")
            .unwrap(),
        None
    );
    assert_eq!(state.try_get::<i64>("", "generation").unwrap(), 0);
    assert_eq!(
        db.query_one_raw(Statement::from_string(
            DbBackend::Postgres,
            "SELECT count(*) AS n FROM lesson_revisions WHERE published"
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap(),
        0
    );

    brioche_server::content::withdraw(
        &db,
        "author-reviewed",
        1,
        0,
        "tester",
        "protocol withdrawal",
    )
    .await
    .unwrap();
    valid["id"] = json!("withdrawn-release");
    let text = write(&release_file, &valid);
    let marker = text.find("\"revision\": 1").unwrap() + "\"revision\": ".len();
    located(
        invoke(url.as_str(), &root, "release-stage", &release_file),
        &release_file,
        &text,
        "/levels/0/units/0/lessons/0/revision",
        marker,
        "was withdrawn",
    );
    assert_eq!(count(&db, "content_releases").await, 1);
    assert_eq!(count(&db, "content_audit").await, 2);
    brioche_migration::Migrator::down(&db, None).await.unwrap();
    drop(db);
    admin
        .execute_unprepared(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    assert!(
        root.canonicalize()
            .unwrap()
            .starts_with(std::env::temp_dir().canonicalize().unwrap())
    );
    std::fs::remove_dir_all(root).unwrap();
}
