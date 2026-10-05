//! Real PostgreSQL registry invariants, isolated from development and production databases.
use axum::{Router, body::Body, http::Request};
use brioche_course_contract::Block;
use brioche_server::{
    audio,
    recording::{self, AudioBundle, AudioSpec},
};
use http_body_util::BodyExt;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DbBackend, Statement};
use sea_orm_migration::MigratorTrait;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;
use tower::ServiceExt;
#[path = "support/assets.rs"]
mod asset_fixtures;

async fn response(
    app: &Router,
    path: &str,
    method: &str,
    headers: &[(&str, &str)],
) -> axum::response::Response {
    let mut request = Request::builder().uri(path).method(method);
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    app.clone()
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap()
}
async fn account(
    app: &Router,
    backend: &brioche_server::identity::Backend,
    email: &str,
    operator: bool,
) -> String {
    let csrf = response(app, "/api/v1/auth/csrf", "GET", &[]).await;
    let cookie = csrf.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let data: Value =
        serde_json::from_slice(&csrf.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let token = backend.issue_token(email, false, operator).await.unwrap();
    let request=Request::builder().method("POST").uri("/api/v1/auth/accept-invite")
        .header("cookie",cookie).header("origin","http://localhost:5173").header("x-csrf-token",data["csrfToken"].as_str().unwrap()).header("content-type","application/json")
        .body(Body::from(serde_json::to_vec(&json!({"email":email,"token":token,"password":"recording protocol only passphrase","displayName":"Protocol account"})).unwrap())).unwrap();
    let result = app.clone().oneshot(request).await.unwrap();
    assert_eq!(result.status(), 200);
    result.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned()
}

fn bundle() -> AudioBundle {
    let bytes = include_bytes!("fixtures/audio/synthetic.mp3");
    AudioBundle {
        schema_version: "1.0".into(),
        assets: vec![AudioSpec {
            asset_id: "audio-protocol".into(),
            revision: 1,
            sha256: format!("{:x}", Sha256::digest(bytes)),
            mime_type: "audio/mpeg".into(),
            duration_ms: 1000,
            credit_zh: "原创合成协议测试音，非课程配音".into(),
            file: "synthetic.mp3".into(),
            status: "ready".into(),
            source: "local FFmpeg sine generator".into(),
            license: "original test fixture; no third-party recording".into(),
            creator: "Brioche protocol tests".into(),
            rights_confirmed: true,
        }],
    }
}

#[tokio::test]
#[ignore = "set TEST_DATABASE_URL to a dedicated PostgreSQL database"]
async fn registration_is_immutable_atomic_and_hydrates_exact_revisions() {
    let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL is required");
    let admin = Database::connect(&url).await.unwrap();
    let mut random = [0u8; 16];
    getrandom::fill(&mut random).unwrap();
    let schema = format!("brioche_audio_test_{}", u128::from_le_bytes(random));
    admin
        .execute_unprepared(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let mut options = ConnectOptions::new(url);
    options.set_schema_search_path(&schema);
    let db = Database::connect(options).await.unwrap();
    brioche_migration::Migrator::up(&db, None).await.unwrap();
    brioche_migration::Migrator::up(&db, None).await.unwrap();
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/audio");
    let store = workspace.join(".local").join(&schema);
    let count = |table: &'static str| {
        let db = db.clone();
        async move {
            db.query_one_raw(Statement::from_string(
                DbBackend::Postgres,
                format!("SELECT count(*)::bigint AS count FROM {table}"),
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get::<i64>("", "count")
            .unwrap()
        }
    };
    for case in 0..5 {
        let mut invalid = bundle();
        match case {
            0 => invalid.assets[0].rights_confirmed = false,
            1 => invalid.assets[0].sha256 = "0".repeat(64),
            2 => invalid.assets[0].duration_ms = 999,
            3 => invalid.assets[0].mime_type = "audio/wav".into(),
            4 => invalid.assets[0].file = "../escape.mp3".into(),
            _ => unreachable!(),
        }
        assert!(
            recording::import_bundle(&db, invalid, &source, &store, "operator-test")
                .await
                .is_err(),
            "case {case}"
        );
    }
    assert_eq!(count("audio_assets").await, 0);
    assert_eq!(count("audio_import_audit").await, 0);
    let (first, second) = tokio::join!(
        recording::import_bundle(&db, bundle(), &source, &store, "operator-test"),
        recording::import_bundle(&db, bundle(), &source, &store, "operator-test")
    );
    assert_ne!(
        first.is_ok(),
        second.is_ok(),
        "exactly one concurrent registration succeeds"
    );
    assert_eq!(count("audio_assets").await, 1);
    assert_eq!(count("audio_import_audit").await, 1);
    let row = db
        .query_one_raw(Statement::from_string(
            DbBackend::Postgres,
            "SELECT * FROM audio_assets",
        ))
        .await
        .unwrap()
        .unwrap();
    let descriptor: Value = row.try_get("", "descriptor").unwrap();
    let provenance: Value = row.try_get("", "provenance").unwrap();
    assert_eq!(provenance["rightsConfirmed"], true);
    assert_eq!(row.try_get::<i32>("", "sample_rate").unwrap(), 24000);
    assert_eq!(row.try_get::<i32>("", "channels").unwrap(), 1);
    assert_eq!(descriptor["durationMs"], 1000);
    for sql in [
        "UPDATE audio_assets SET duration_ms=999",
        "DELETE FROM audio_assets",
        "UPDATE audio_import_audit SET actor='tampered'",
        "DELETE FROM audio_import_audit",
    ] {
        assert!(db.execute_unprepared(sql).await.is_err(), "{sql}");
    }
    let mut mixed = bundle();
    let mut new = mixed.assets[0].clone();
    new.asset_id = "new-but-rolled-back".into();
    mixed.assets.insert(0, new);
    assert!(
        recording::import_bundle(&db, mixed, &source, &store, "operator-test")
            .await
            .is_err()
    );
    assert_eq!(count("audio_assets").await, 1);
    assert_eq!(count("audio_import_audit").await, 1);
    let mut second_revision = bundle();
    second_revision.assets[0].revision = 2;
    second_revision.assets[0].credit_zh = "第二个不可变版本".into();
    recording::import_bundle(&db, second_revision, &source, &store, "operator-test")
        .await
        .unwrap();
    let visual_store = asset_fixtures::fixture_assets(&db, &schema).await;
    for entry in std::fs::read_dir(&visual_store).unwrap() {
        let entry = entry.unwrap();
        std::fs::copy(entry.path(), store.join(entry.file_name())).unwrap();
    }
    let mut source_doc = brioche_server::development_source().unwrap();
    source_doc["assetRefs"] = asset_fixtures::fixture_refs();
    source_doc = brioche_server::media::hydrate_source(&db, source_doc)
        .await
        .unwrap();
    source_doc["audioRefs"] = json!([{"assetId":"audio-protocol","revision":1}]);
    source_doc["audio"] = json!([{"forged":true}]);
    let mut hydrated = recording::hydrate_source(&db, source_doc).await.unwrap();
    assert_eq!(hydrated["audio"], json!([descriptor]));
    let Block::Dialogue { id, turns, .. } =
        &brioche_server::development_fixture().unwrap().blocks[1]
    else {
        panic!()
    };
    let cues: Vec<Value> = turns
        .iter()
        .enumerate()
        .map(|(i, turn)| json!({"entryId":turn.id,"startMs":i*100,"endMs":(i+1)*100}))
        .collect();
    hydrated["audioTracks"] = json!([{"blockId":id,"assetId":"audio-protocol","cues":cues}]);
    let lesson = brioche_server::project_source(hydrated.clone()).unwrap();
    assert_eq!(lesson.audio[0].revision, 1);
    assert!(
        !serde_json::to_value(&lesson)
            .unwrap()
            .to_string()
            .contains("audioRefs")
    );
    assert!(
        !serde_json::to_value(&lesson)
            .unwrap()
            .to_string()
            .contains("license")
    );
    brioche_server::media::validate_lesson(&db, &lesson, &store)
        .await
        .unwrap();
    let mut forged = lesson.clone();
    forged.audio[0].credit_zh = "tampered credit".into();
    assert!(
        recording::validate_lesson(&db, &forged, &store)
            .await
            .is_err()
    );
    // A protocol-only assertion exercises publication gates, not human language review.
    hydrated["editorial"] = json!({"status":"reviewed","note":"Protocol fixture assertion; no French pronunciation or language review claimed"});
    db.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"INSERT INTO lesson_revisions(lesson_id,revision,published,public_document,server_document) VALUES($1,$2,false,$3,$4)", [lesson.id.clone().into(),(lesson.revision as i32).into(),serde_json::to_value(&lesson).unwrap().into(),hydrated.clone().into()])).await.unwrap();
    let manifest = brioche_server::content::ReleaseManifest {
        id: "audio-test-release".into(),
        schema_version: "1.0".into(),
        levels: vec![brioche_server::content::ReleaseLevel {
            id: lesson.level_id.clone(),
            label: "Protocol".into(),
            units: vec![brioche_server::content::ReleaseUnit {
                id: lesson.unit_id.clone(),
                title_zh: "协议测试".into(),
                lessons: vec![brioche_server::content::RevisionRef {
                    lesson_id: lesson.id.clone(),
                    revision: lesson.revision,
                }],
            }],
        }],
    };
    let backend = brioche_server::identity::Backend::new(db.clone())
        .await
        .unwrap();
    let auth = brioche_server::identity::router_with_media_root(
        backend.clone(),
        brioche_server::csrf::CsrfPolicy::new(["http://localhost:5173".into()]).unwrap(),
        false,
        store.clone(),
    );
    let app = recording::router(db.clone(), store.clone()).merge(auth);
    let url = lesson.audio[0].url.clone();
    let preview = format!("/api/v1/operator/lessons/{}/revisions/1", lesson.id);
    let private_url = format!("{preview}/audio/{}", url.rsplit('/').next().unwrap());
    assert_eq!(
        response(&app, &url, "GET", &[]).await.status(),
        404,
        "registration is private"
    );
    assert_eq!(response(&app, &private_url, "GET", &[]).await.status(), 401);
    let learner = account(&app, &backend, "audio-learner@example.test", false).await;
    let operator = account(&app, &backend, "audio-operator@example.test", true).await;
    assert_eq!(
        response(&app, &private_url, "GET", &[("cookie", &learner)])
            .await
            .status(),
        403
    );
    let draft = response(&app, &preview, "GET", &[("cookie", &operator)]).await;
    assert_eq!(draft.status(), 200);
    let draft: Value =
        serde_json::from_slice(&draft.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(draft["audio"][0]["url"], private_url);
    let clip = response(
        &app,
        &private_url,
        "GET",
        &[("cookie", &operator), ("range", "bytes=0-9")],
    )
    .await;
    assert_eq!(clip.status(), 206);
    assert_eq!(clip.headers()["cache-control"], "private, no-store");
    assert_eq!(
        clip.into_body().collect().await.unwrap().to_bytes().len(),
        10
    );
    assert_eq!(
        response(
            &app,
            &format!("{preview}/audio/{}.mp3", "a".repeat(64)),
            "GET",
            &[("cookie", &operator)]
        )
        .await
        .status(),
        404
    );
    db.execute_unprepared(
        "UPDATE users SET role='learner' WHERE email='audio-operator@example.test'",
    )
    .await
    .unwrap();
    assert_eq!(
        response(&app, &private_url, "GET", &[("cookie", &operator)])
            .await
            .status(),
        403,
        "role is rechecked on each audio request"
    );
    db.execute_unprepared(
        "UPDATE users SET role='operator' WHERE email='audio-operator@example.test'",
    )
    .await
    .unwrap();
    let mut other = lesson.clone();
    other.id = "a1-other-protocol".into();
    other.audio.clear();
    other.audio_tracks.clear();
    let other_id = other.id.clone();
    db.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"INSERT INTO lesson_revisions(lesson_id,revision,published,public_document,server_document) VALUES($1,1,false,$2,$3)",[other_id.into(),serde_json::to_value(other).unwrap().into(),json!({}).into()])).await.unwrap();
    assert_eq!(
        response(
            &app,
            &format!(
                "/api/v1/operator/lessons/a1-other-protocol/revisions/1/audio/{}",
                url.rsplit('/').next().unwrap()
            ),
            "GET",
            &[("cookie", &operator)]
        )
        .await
        .status(),
        404,
        "private audio belongs to the selected course"
    );
    brioche_server::content::stage(&db, &manifest, "protocol-test", "audio gate", &store)
        .await
        .unwrap();
    assert_eq!(
        response(&app, &url, "GET", &[]).await.status(),
        404,
        "staging remains private"
    );
    let generation = brioche_server::content::activate(
        &db,
        &manifest.id,
        0,
        "protocol-test",
        "audio gate",
        &store,
    )
    .await
    .unwrap();
    let full = response(&app, &url, "GET", &[]).await;
    assert_eq!(full.status(), 200);
    assert_eq!(full.headers()["content-type"], "audio/mpeg");
    assert_eq!(full.headers()["cache-control"], "no-store");
    let etag = full.headers()["etag"].to_str().unwrap().to_owned();
    let bytes = full.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&bytes[..], include_bytes!("fixtures/audio/synthetic.mp3"));
    for (range, start, end) in [
        ("bytes=2-9", 2, 10),
        ("bytes=20-", 20, bytes.len()),
        ("bytes=-5", bytes.len() - 5, bytes.len()),
        ("bytes=0-999999", 0, bytes.len()),
    ] {
        let partial = response(&app, &url, "GET", &[("range", range)]).await;
        assert_eq!(partial.status(), 206);
        assert_eq!(
            partial.headers()["content-range"],
            format!("bytes {start}-{}/{}", end - 1, bytes.len())
        );
        assert_eq!(
            partial.into_body().collect().await.unwrap().to_bytes(),
            bytes.slice(start..end)
        );
    }
    assert_eq!(
        response(
            &app,
            &url,
            "GET",
            &[("range", "bytes=0-1"), ("range", "bytes=2-3")]
        )
        .await
        .status(),
        416
    );
    for range in ["bytes=999999-", "bytes=2-1", "bytes=0-1,2-3", "bytes=-0"] {
        let invalid = response(&app, &url, "GET", &[("range", range)]).await;
        assert_eq!(invalid.status(), 416);
        assert_eq!(
            invalid.headers()["content-range"],
            format!("bytes */{}", bytes.len())
        );
    }
    assert_eq!(
        response(
            &app,
            &url,
            "GET",
            &[("range", "bytes=0-1"), ("if-range", &etag)]
        )
        .await
        .status(),
        206
    );
    assert_eq!(
        response(
            &app,
            &url,
            "GET",
            &[("range", "bytes=0-1"), ("if-range", "\"old\"")]
        )
        .await
        .status(),
        200
    );
    let head = response(&app, &url, "HEAD", &[]).await;
    assert_eq!(head.status(), 200);
    assert_eq!(head.headers()["content-length"], bytes.len().to_string());
    assert!(
        head.into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .is_empty()
    );
    hydrated["audioRefs"] = json!([{"assetId":"audio-protocol","revision":2}]);
    let second_hydrated = recording::hydrate_source(&db, hydrated.clone())
        .await
        .unwrap();
    assert_eq!(second_hydrated["audio"][0]["creditZh"], "第二个不可变版本");
    hydrated["audioRefs"] = json!([{"assetId":"audio-protocol","revision":3}]);
    assert!(recording::hydrate_source(&db, hydrated).await.is_err());
    let object = store.join(format!("{}.mp3", bundle().assets[0].sha256));
    assert_eq!(
        audio::inspect_file(&object, "audio/mpeg")
            .unwrap()
            .1
            .duration_ms,
        1000
    );
    std::fs::write(&object, b"corrupt").unwrap();
    assert!(
        recording::validate_lesson(&db, &lesson, &store)
            .await
            .is_err()
    );
    assert_eq!(
        response(&app, &url, "GET", &[("range", "bytes=0-1")])
            .await
            .status(),
        503
    );
    assert_eq!(
        response(&app, &private_url, "GET", &[("cookie", &operator)])
            .await
            .status(),
        503
    );
    let mut third = bundle();
    third.assets[0].revision = 3;
    assert!(
        recording::import_bundle(&db, third, &source, &store, "operator-test")
            .await
            .is_err()
    );
    assert_eq!(count("audio_assets").await, 2);
    assert_eq!(count("audio_import_audit").await, 2);
    std::fs::write(&object, &bytes).unwrap();
    assert_eq!(response(&app, &url, "GET", &[]).await.status(), 200);
    brioche_server::content::withdraw(
        &db,
        &lesson.id,
        1,
        generation,
        "protocol-test",
        "withdraw audio",
    )
    .await
    .unwrap();
    assert_eq!(response(&app, &url, "GET", &[]).await.status(), 404);
    assert_eq!(
        response(&app, &private_url, "GET", &[("cookie", &operator)])
            .await
            .status(),
        410
    );
    assert_eq!(
        response(&app, &private_url, "GET", &[("cookie", &learner)])
            .await
            .status(),
        403
    );
    brioche_migration::Migrator::down(&db, None).await.unwrap();
    db.close().await.unwrap();
    admin
        .execute_unprepared(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    let resolved_store = store.canonicalize().unwrap();
    assert!(resolved_store.starts_with(workspace.join(".local")));
    std::fs::remove_dir_all(resolved_store).unwrap();
    let resolved_visual = visual_store.canonicalize().unwrap();
    assert!(resolved_visual.starts_with(std::env::temp_dir().canonicalize().unwrap()));
    std::fs::remove_dir_all(resolved_visual).unwrap();
}
