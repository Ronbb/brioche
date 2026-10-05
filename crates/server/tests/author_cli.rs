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
    source = original.clone();
    source["audioRefs"] =
        serde_json::json!([{"assetId":"audio","revision":1},{"assetId":"audio","revision":2}]);
    std::fs::write(&path, serde_json::to_vec(&source).unwrap()).unwrap();
    let output = run("check", &path);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("audioRefs/1"));
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

#[test]
fn audio_check_decodes_without_database_and_reports_actual_duration() {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/audio/synthetic.mp3");
    let output = Command::new(env!("CARGO_BIN_EXE_brioche-server"))
        .args(["audio-check", file.to_str().unwrap(), "audio/mpeg"])
        .env("DATABASE_URL", "postgres://invalid@127.0.0.1:1/unavailable")
        .env("CONTENT_MODE", "fixture")
        .env("APP_ENV", "production")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let info: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(info["durationMs"], 1000);
    assert_eq!(info["channels"], 1);
    assert_eq!(info["sha256"].as_str().unwrap().len(), 64);
    let output = Command::new(env!("CARGO_BIN_EXE_brioche-server"))
        .args(["audio-check", file.to_str().unwrap(), "audio/wav"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not RIFF WAVE"));
    assert!(!run("audio-check", &file).status.success());
}

#[test]
fn semantic_and_projected_type_errors_point_into_original_author_source() {
    let path = std::env::temp_dir().join(format!("brioche-locations-{}.json", random_id()));
    let original = brioche_server::development_source().unwrap();
    let dialogue_index = original["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .position(|block| block["type"] == "dialogue")
        .unwrap();
    for (pointer, marker) in [
        (
            format!("/blocks/{dialogue_index}/turns/0/segments/0/vocabularyId"),
            "missing-vocabulary",
        ),
        ("/steps/0/blockIds/0".into(), "missing-block"),
        ("/reviewItemIds/0".into(), "missing-review"),
        ("/completion/requiredStepIds/0".into(), "missing-step"),
        ("/revision".into(), "not-a-revision"),
        ("/editorial/status".into(), "unknown-status"),
    ] {
        let mut source = original.clone();
        *source.pointer_mut(&pointer).unwrap() = serde_json::json!(marker);
        let text = serde_json::to_string_pretty(&source).unwrap();
        std::fs::write(&path, &text).unwrap();
        let offset = text.find(&format!("\"{marker}\"")).unwrap();
        let before = &text[..offset];
        let line = before.bytes().filter(|b| *b == b'\n').count() + 1;
        let column = before.rsplit('\n').next().unwrap().chars().count() + 1;
        let output = run("check", &path);
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success());
        assert!(
            error.contains(&format!("{}:{line}:{column}: {pointer}: ", path.display())),
            "{error}"
        );
        assert!(!error.contains("database connection"));
    }
    let mut source = original;
    source["audioRefs"] =
        serde_json::json!([{"assetId":"test","revision":"invalid-audio-revision"}]);
    let text = serde_json::to_string_pretty(&source).unwrap();
    std::fs::write(&path, &text).unwrap();
    let output = run("check", &path);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("/audioRefs/0/revision:"));
    source.as_object_mut().unwrap().remove("audioRefs");
    let sha = "a".repeat(64);
    source["audio"] = serde_json::json!([{
        "assetId":"test-recording", "revision":1, "sha256":sha,
        "mimeType":"audio/wav", "durationMs":1000, "creditZh":"Protocol test",
        "url":format!("/api/audio/{sha}.wav")
    }]);
    source["audioTracks"] = serde_json::json!([{
        "blockId":source["blocks"][dialogue_index]["id"], "assetId":"test-recording",
        "cues":[{"entryId":source["blocks"][dialogue_index]["turns"][0]["id"],"startMs":0,"endMs":1500}]
    }]);
    let text = serde_json::to_string_pretty(&source).unwrap();
    std::fs::write(&path, &text).unwrap();
    let end_field = text.find("\"endMs\": 1500").unwrap();
    let cue_start = text[..end_field].rfind('{').unwrap();
    let before = &text[..cue_start];
    let line = before.bytes().filter(|b| *b == b'\n').count() + 1;
    let column = before.rsplit('\n').next().unwrap().chars().count() + 1;
    let output = run("check", &path);
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        error.contains(&format!(
            "{}:{line}:{column}: /audioTracks/0/cues/0:",
            path.display()
        )),
        "{error}"
    );
    assert!(error.contains("interval outside recording duration"));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn private_rules_and_release_semantics_report_exact_source_fields() {
    let path = std::env::temp_dir().join(format!("brioche-author-rules-{}.json", random_id()));
    let lesson = brioche_server::development_source().unwrap();
    let release: serde_json::Value =
        serde_json::from_str(include_str!("../../../docs/examples/catalog.release.json")).unwrap();
    for (command, original, pointer, marker) in [
        (
            "check",
            &lesson,
            "/serverOnly/grading/exercise-intention/correctOptionId",
            "private-unknown-option",
        ),
        (
            "check",
            &lesson,
            "/serverOnly/grading/exercise-order/correctTokenIds/1",
            "private-unknown-token",
        ),
        (
            "check",
            &lesson,
            "/serverOnly/grading/exercise-article/accepted/0",
            "\u{00a0}",
        ),
        (
            "check-release",
            &release,
            "/schemaVersion",
            "unsupported-release-version",
        ),
        (
            "check-release",
            &release,
            "/levels/0/units/0/lessons/0/lessonId",
            "invalid lesson reference",
        ),
        (
            "check-release",
            &release,
            "/levels/0/units/0/lessons/0/revision",
            "invalid-release-revision",
        ),
    ] {
        let mut source = original.clone();
        *source.pointer_mut(pointer).unwrap() = serde_json::json!(marker);
        let text = serde_json::to_string_pretty(&source).unwrap();
        std::fs::write(&path, &text).unwrap();
        let token = serde_json::to_string(marker).unwrap();
        let offset = text.find(&token).unwrap();
        let before = &text[..offset];
        let line = before.bytes().filter(|b| *b == b'\n').count() + 1;
        let column = before.rsplit('\n').next().unwrap().chars().count() + 1;
        let output = run(command, &path);
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success());
        assert!(
            error.contains(&format!("{}:{line}:{column}: {pointer}:", path.display())),
            "{error}"
        );
        if command == "check" {
            assert!(!error.contains(marker));
        }
        assert!(!error.contains("database connection"));
    }
    std::fs::remove_file(path).unwrap();
}
