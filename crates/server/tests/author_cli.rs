use std::{
    path::Path,
    process::{Command, Output},
};

fn random_id() -> u128 {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).unwrap();
    u128::from_le_bytes(bytes)
}

fn run(command: &str, path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_brioche-server"))
        .args([command, path.to_str().unwrap()])
        // A deliberately unusable connection proves author checks never connect.
        .env("DATABASE_URL", "postgres://invalid@127.0.0.1:1/unavailable")
        .env("CONTENT_MODE", "fixture")
        .env("APP_ENV", "production")
        .output()
        .unwrap()
}

#[test]
fn checks_drafts_without_database_and_does_not_claim_publication() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/examples");
    for (command, file) in [
        ("check", "a1-bakery.lesson.json"),
        ("check-release", "catalog.release.json"),
    ] {
        let output = run(command, &root.join(file));
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("Publication still requires"));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("correctOptionId"));
    }
}

#[test]
fn rejects_bad_grading_references_and_json_before_any_database_work() {
    let path = std::env::temp_dir().join(format!("brioche-check-{}.json", random_id()));
    let original = brioche_server::development_source().unwrap();
    let mut source = original.clone();
    source["serverOnly"]["grading"] = serde_json::json!({});
    std::fs::write(&path, serde_json::to_vec(&source).unwrap()).unwrap();
    let output = run("check", &path);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("serverOnly.grading"));
    source = original.clone();
    source["assetRefs"] =
        serde_json::json!([{"assetId":"scene", "revision":1},{"assetId":"scene", "revision":2}]);
    std::fs::write(&path, serde_json::to_vec(&source).unwrap()).unwrap();
    let output = run("check", &path);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("assetRefs/1"));
    source = original;
    source["revision"] = serde_json::json!("invalid");
    std::fs::write(&path, serde_json::to_vec(&source).unwrap()).unwrap();
    let output = run("check", &path);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("revision"));
    std::fs::write(&path, br#"{"id":"a","id":"b"}"#).unwrap();
    let output = run("check", &path);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicate JSON field"));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn release_check_rejects_invalid_manifest_and_extra_arguments() {
    let path = std::env::temp_dir().join(format!("brioche-check-{}.json", random_id()));
    std::fs::write(
        &path,
        br#"{"id":"release","schemaVersion":"unknown","levels":[]}"#,
    )
    .unwrap();
    assert!(!run("check-release", &path).status.success());
    std::fs::remove_file(path).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_brioche-server"))
        .args(["check", "one.json", "--publish"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage:"));
}
