//! Real operator authorization and editorial decisions against a disposable schema.
use axum::{Router, body::Body, http::Request};
use brioche_server::{
    csrf::CsrfPolicy,
    identity::{self, Backend},
};
use http_body_util::BodyExt;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DbBackend, Statement};
use sea_orm_migration::MigratorTrait;
use serde_json::{Value, json};
use tower::ServiceExt;
#[path = "support/assets.rs"]
mod assets;
struct Browser {
    app: Router,
    cookie: String,
    csrf: String,
}
#[derive(Default)]
struct MockQwen {
    calls: std::sync::Mutex<Vec<Value>>,
    status: std::sync::Mutex<String>,
    unknown: std::sync::atomic::AtomicBool,
}
#[async_trait::async_trait]
impl brioche_server::qwen::Transport for MockQwen {
    async fn create(
        &self,
        prefix: &str,
        url: &str,
    ) -> Result<brioche_server::qwen::Receipt, brioche_server::qwen::ProviderError> {
        self.calls
            .lock()
            .unwrap()
            .push(json!({"create":prefix,"url":url}));
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if self.unknown.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(brioche_server::qwen::ProviderError::Unknown);
        }
        Ok(brioche_server::qwen::Receipt {
            voice_id: format!("{}-{prefix}-test", brioche_server::qwen::MODEL),
            request_id: "create-test".into(),
        })
    }
    async fn query(
        &self,
        voice: &str,
    ) -> Result<brioche_server::qwen::Details, brioche_server::qwen::ProviderError> {
        self.calls.lock().unwrap().push(json!({"query":voice}));
        let status = self.status.lock().unwrap().clone();
        Ok(brioche_server::qwen::Details {
            model: if status == "mismatch" {
                "other-model".into()
            } else {
                brioche_server::qwen::MODEL.into()
            },
            status: if status == "mismatch" {
                "OK".into()
            } else {
                status
            },
            request_id: "query-test".into(),
        })
    }
    async fn synthesize(
        &self,
        request: &brioche_server::qwen::SpeechRequest,
    ) -> Result<brioche_server::qwen::Speech, brioche_server::qwen::ProviderError> {
        self.calls
            .lock()
            .unwrap()
            .push(json!({"synthesis":request.parameters().unwrap()}));
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if self.unknown.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(brioche_server::qwen::ProviderError::Unknown);
        }
        let mut wav = vec![0u8; 4844];
        wav[..4].copy_from_slice(b"RIFF");
        wav[4..8].copy_from_slice(&4836u32.to_le_bytes());
        wav[8..16].copy_from_slice(b"WAVEfmt ");
        wav[16..20].copy_from_slice(&16u32.to_le_bytes());
        wav[20..24].copy_from_slice(&[1, 0, 1, 0]);
        wav[24..28].copy_from_slice(&24000u32.to_le_bytes());
        wav[28..32].copy_from_slice(&48000u32.to_le_bytes());
        wav[32..36].copy_from_slice(&[2, 0, 16, 0]);
        wav[36..40].copy_from_slice(b"data");
        wav[40..44].copy_from_slice(&4800u32.to_le_bytes());
        let info = brioche_server::audio::inspect(&wav, "audio/wav").unwrap();
        Ok(brioche_server::qwen::Speech {
            provider_wav: wav.clone(),
            wav,
            info,
            request_id: "audition-test".into(),
            verification: Some(brioche_server::qwen::Details {
                model: brioche_server::qwen::MODEL.into(),
                status: "OK".into(),
                request_id: "verify-audition".into(),
            }),
            input_tokens: Some(10),
            output_tokens: Some(20),
        })
    }
}
async fn settled(browser: &mut Browser, path: &str) -> Value {
    for _ in 0..100 {
        let read = browser.send("GET", path, None, true).await;
        assert_eq!(read.0, 200);
        if !["submitted", "checking"].contains(&read.1["status"].as_str().unwrap()) {
            return read.1;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("owned provider test worker did not settle");
}

#[tokio::test]
#[ignore = "set TEST_DATABASE_URL to a dedicated PostgreSQL database"]
async fn voice_reference_delivery_is_bounded_revocable_private_and_audited() {
    let url = std::env::var("TEST_DATABASE_URL").unwrap();
    let admin = Database::connect(&url).await.unwrap();
    let schema = format!(
        "voice_reference_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    admin
        .execute_unprepared(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let mut options = ConnectOptions::new(url);
    options.set_schema_search_path(&schema);
    let db = Database::connect(options).await.unwrap();
    brioche_migration::Migrator::up(&db, None).await.unwrap();
    let root = assets::fixture_assets(&db, &schema).await;
    let backend = Backend::new(db.clone()).await.unwrap();
    let base_app = identity::router_with_media_root(
        backend.clone(),
        CsrfPolicy::new(["http://localhost:5173".into()]).unwrap(),
        false,
        root.clone(),
    );
    let qwen = std::sync::Arc::new(MockQwen::default());
    *qwen.status.lock().unwrap() = "OK".into();
    let app = base_app.clone().layer(axum::Extension(
        brioche_server::qwen::Service::new(qwen.clone(), "https://example.test").unwrap(),
    ));
    let mut visitor = Browser::new(app.clone()).await;
    let mut operator = Browser::new(app.clone()).await;
    operator
        .register(&backend, "reference-operator@example.test", true)
        .await;
    let mut learner = Browser::new(app.clone()).await;
    learner
        .register(&backend, "reference-learner@example.test", false)
        .await;
    // Five seconds of synthetic PCM solely for protocol validation, no real speaker/consent claim.
    let mut wav = vec![0u8; 160044];
    wav[..4].copy_from_slice(b"RIFF");
    wav[4..8].copy_from_slice(&(160036u32).to_le_bytes());
    wav[8..12].copy_from_slice(b"WAVE");
    wav[12..16].copy_from_slice(b"fmt ");
    wav[16..20].copy_from_slice(&16u32.to_le_bytes());
    wav[20..22].copy_from_slice(&1u16.to_le_bytes());
    wav[22..24].copy_from_slice(&1u16.to_le_bytes());
    wav[24..28].copy_from_slice(&16000u32.to_le_bytes());
    wav[28..32].copy_from_slice(&32000u32.to_le_bytes());
    wav[32..34].copy_from_slice(&2u16.to_le_bytes());
    wav[34..36].copy_from_slice(&16u16.to_le_bytes());
    wav[36..40].copy_from_slice(b"data");
    wav[40..44].copy_from_slice(&160000u32.to_le_bytes());
    let upload = json!({"assetId":"qa-reference-delivery","revision":1,"mimeType":"audio/wav","creditZh":"合成协议测试","source":"test:synthetic","license":"LicenseRef-TestOnly","creator":"test fixture","rightsConfirmed":true,"reason":"isolated reference file"});
    assert_eq!(
        operator
            .upload_media("/api/v1/operator/recordings", upload.clone(), &wav, true)
            .await
            .0,
        200
    );
    let seed: Value =
        serde_json::from_str(include_str!("../../../docs/characters/voices.json")).unwrap();
    let mut profile = seed["items"][0]["profile"].clone();
    profile["referenceAudio"] = json!({"assetId":"qa-reference-delivery","revision":1,"transcript":"Synthetic five second fixture","cloningPermission":"No real person; protocol test only"});
    let voice = json!({"characterId":"character-camille","characterRevision":1,"expectedVoiceRevision":0,"profile":profile,"reason":"isolated voice reference"});
    assert_eq!(
        operator
            .send("POST", "/api/v1/operator/characters", Some(voice), true)
            .await
            .0,
        200
    );
    let path = "/api/v1/operator/voice-references";
    let request = json!({"characterId":"character-camille","characterRevision":1,"voiceRevision":1,"singleSpeakerConfirmed":true,"reason":"isolated authorized delivery"});
    assert_eq!(visitor.send("GET", path, None, true).await.0, 401);
    assert_eq!(learner.send("GET", path, None, true).await.0, 403);
    assert_eq!(
        learner
            .send("POST", path, Some(request.clone()), true)
            .await
            .0,
        403
    );
    assert_eq!(
        operator
            .send("POST", path, Some(request.clone()), false)
            .await
            .0,
        403
    );
    let mut unconfirmed = request.clone();
    unconfirmed["singleSpeakerConfirmed"] = json!(false);
    assert_eq!(
        operator.send("POST", path, Some(unconfirmed), true).await.0,
        400
    );
    let mut missing = request.clone();
    missing["voiceRevision"] = json!(99);
    assert_eq!(
        operator.send("POST", path, Some(missing), true).await.0,
        404
    );
    let (status, result) = operator
        .send("POST", path, Some(request.clone()), true)
        .await;
    assert_eq!(status, 200);
    let grant_id = result["grant"]["id"].as_str().unwrap();
    let bearer = result["path"].as_str().unwrap();
    let token = bearer.rsplit('/').next().unwrap();
    assert_eq!(token.len(), 64);
    assert_eq!(
        operator
            .send("POST", path, Some(request.clone()), true)
            .await
            .0,
        409
    );
    let list = operator.send("GET", path, None, true).await;
    assert_eq!(list.1["items"][0]["assetRevision"], 1);
    assert!(!list.1.to_string().contains(token));
    assert!(!list.1.to_string().contains("tokenHash"));
    assert!(!list.1.to_string().contains("cloningPermission"));
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(bearer)
                .header("range", "bytes=0-9")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 206);
    assert_eq!(response.headers()["cache-control"], "private, no-store");
    assert_eq!(response.headers()["x-content-type-options"], "nosniff");
    assert_eq!(
        response.into_body().collect().await.unwrap().to_bytes(),
        &wav[..10]
    );
    let wrong = format!("/api/v1/voice-references/{grant_id}/{}", "0".repeat(64));
    assert_eq!(visitor.send("GET", &wrong, None, false).await.0, 404);
    assert_eq!(
        operator.send("GET", path, None, true).await.1["items"][0]["readCount"],
        1
    );
    let expired_id = "e".repeat(32);
    let expired_token = "e".repeat(64);
    use sha2::Digest;
    let expired_hash = format!("{:x}", sha2::Sha256::digest(expired_token.as_bytes()));
    db.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"INSERT INTO voice_reference_grants SELECT $1,$2,character_id,character_revision,voice_revision,asset_id,asset_revision,descriptor,reference,actor_id,reason,model,single_speaker_confirmed,CURRENT_TIMESTAMP-interval '30 minutes',CURRENT_TIMESTAMP-interval '20 minutes' FROM voice_reference_grants WHERE id=$3",vec![expired_id.clone().into(),expired_hash.into(),grant_id.into()])).await.unwrap();
    assert_eq!(
        visitor
            .send(
                "GET",
                &format!("/api/v1/voice-references/{expired_id}/{expired_token}"),
                None,
                false
            )
            .await
            .0,
        404
    );
    let stored = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT token_hash,reference,actor_id FROM voice_reference_grants WHERE id=$1",
            vec![grant_id.into()],
        ))
        .await
        .unwrap()
        .unwrap();
    assert_ne!(stored.try_get::<String>("", "token_hash").unwrap(), token);
    assert_eq!(
        stored.try_get::<Value>("", "reference").unwrap()["cloningPermission"],
        profile["referenceAudio"]["cloningPermission"]
    );
    let revoke = format!("{path}/{grant_id}/revoke");
    assert_eq!(
        learner
            .send("POST", &revoke, Some(json!({"reason":"test"})), true)
            .await
            .0,
        403
    );
    assert_eq!(
        operator
            .send("POST", &revoke, Some(json!({"reason":"test"})), false)
            .await
            .0,
        403
    );
    assert_eq!(
        operator
            .send("POST", &revoke, Some(json!({"reason":"test"})), true)
            .await
            .0,
        200
    );
    assert_eq!(visitor.send("GET", bearer, None, false).await.0, 404);
    assert_eq!(
        operator
            .send("POST", &revoke, Some(json!({"reason":"test"})), true)
            .await
            .0,
        409
    );
    assert!(
        brioche_migration::Migrator::migrations()
            .into_iter()
            .find(|m| m.name() == "m20261007_000018_voice_reference_grants")
            .unwrap()
            .down(&sea_orm_migration::SchemaManager::new(&db))
            .await
            .is_err()
    );
    // Account lock serializes simultaneous issuance: one credential, one explicit conflict.
    let parallel_cookie = operator.cookie.clone();
    let parallel_csrf = operator.csrf.clone();
    let (a, b) = tokio::join!(
        operator.send("POST", path, Some(request.clone()), true),
        learner.app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("cookie", &parallel_cookie)
                .header("origin", "http://localhost:5173")
                .header("x-csrf-token", &parallel_csrf)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap()
        )
    );
    let response = b.unwrap();
    let b_status = response.status().as_u16();
    let b_result: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let mut statuses = vec![a.0, b_status];
    statuses.sort();
    assert_eq!(statuses, vec![200, 409]);
    let next = if a.0 == 200 { a.1 } else { b_result };
    let next_id = next["grant"]["id"].as_str().unwrap();
    let next_bearer = next["path"].as_str().unwrap();
    // Exhaustion is atomic and applies equally to HEAD/Range retries.
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO voice_reference_reads(grant_id) SELECT $1 FROM generate_series(1,32)",
        vec![next_id.into()],
    ))
    .await
    .unwrap();
    assert_eq!(visitor.send("GET", next_bearer, None, false).await.0, 404);
    let third_revoke = format!("{path}/{next_id}/revoke");
    assert_eq!(
        operator
            .send(
                "POST",
                &third_revoke,
                Some(json!({"reason":"after exhaustion"})),
                true
            )
            .await
            .0,
        200
    );
    let third = operator
        .send("POST", path, Some(request.clone()), true)
        .await;
    assert_eq!(third.0, 200);
    let third_path = third.1["path"].as_str().unwrap();
    let actor = stored.try_get::<i64>("", "actor_id").unwrap();
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE users SET role='learner' WHERE id=$1",
        vec![actor.into()],
    ))
    .await
    .unwrap();
    assert_eq!(visitor.send("GET", third_path, None, false).await.0, 404);
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE users SET role='operator' WHERE id=$1",
        vec![actor.into()],
    ))
    .await
    .unwrap();
    let sha = format!("{:x}", sha2::Sha256::digest(&wav));
    let file = root.join(format!("{sha}.wav"));
    assert!(file.exists());
    std::fs::write(&file, b"corrupt").unwrap();
    assert_eq!(visitor.send("GET", third_path, None, false).await.0, 400);
    std::fs::write(&file, &wav).unwrap();
    // Registered audio is not enough: the provider's actual minimum duration is rechecked.
    let mut short = wav[..32044].to_vec();
    short[4..8].copy_from_slice(&32036u32.to_le_bytes());
    short[40..44].copy_from_slice(&32000u32.to_le_bytes());
    let mut short_upload = upload;
    short_upload["revision"] = json!(2);
    assert_eq!(
        operator
            .upload_media("/api/v1/operator/recordings", short_upload, &short, true)
            .await
            .0,
        200
    );
    profile["referenceAudio"]["revision"] = json!(2);
    let short_voice = json!({"characterId":"character-camille","characterRevision":1,"expectedVoiceRevision":1,"profile":profile,"reason":"short reference test"});
    assert_eq!(
        operator
            .send(
                "POST",
                "/api/v1/operator/characters",
                Some(short_voice),
                true
            )
            .await
            .0,
        200
    );
    let mut short_request = request;
    short_request["voiceRevision"] = json!(2);
    assert_eq!(
        operator
            .send("POST", path, Some(short_request), true)
            .await
            .0,
        400
    );
    for i in 0..25u32 {
        db.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"INSERT INTO voice_reference_grants SELECT $1,$2,character_id,character_revision,voice_revision,asset_id,asset_revision,descriptor,reference,actor_id,reason,model,single_speaker_confirmed,CURRENT_TIMESTAMP-interval '30 minutes',CURRENT_TIMESTAMP-interval '20 minutes' FROM voice_reference_grants WHERE id=$3",vec![format!("{i:032x}").into(),format!("{i:064x}").into(),grant_id.into()])).await.unwrap();
    }
    let first = operator.send("GET", path, None, true).await.1;
    assert_eq!(first["items"].as_array().unwrap().len(), 20);
    let cursor = first["next"].as_str().unwrap();
    let second = operator
        .send("GET", &format!("{path}?afterId={cursor}"), None, true)
        .await
        .1;
    assert_eq!(second["items"].as_array().unwrap().len(), 9);
    let ids = first["items"]
        .as_array()
        .unwrap()
        .iter()
        .chain(second["items"].as_array().unwrap())
        .map(|g| g["id"].as_str().unwrap())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(ids.len(), 29);
    // Audit tables cannot be edited, including expiry, token and consent.
    assert!(
        db.execute_unprepared("UPDATE voice_reference_grants SET reason='overwrite'")
            .await
            .is_err()
    );
    assert!(
        db.execute_unprepared("DELETE FROM voice_reference_revocations")
            .await
            .is_err()
    );
    assert!(
        db.execute_unprepared("DELETE FROM voice_reference_reads")
            .await
            .is_err()
    );
    let jobs = "/api/v1/operator/voice-jobs";
    let create = json!({"grantId":third.1["grant"]["id"],"token":third_path.rsplit('/').next().unwrap(),"costConfirmed":true,"reason":"isolated enrollment"});
    assert_eq!(visitor.send("GET", jobs, None, true).await.0, 401);
    assert_eq!(learner.send("GET", jobs, None, true).await.0, 403);
    assert_eq!(
        operator
            .send("POST", jobs, Some(create.clone()), false)
            .await
            .0,
        403
    );
    assert_eq!(
        learner
            .send("POST", jobs, Some(create.clone()), true)
            .await
            .0,
        403
    );
    let mut disabled = Browser {
        app: base_app,
        cookie: operator.cookie.clone(),
        csrf: operator.csrf.clone(),
    };
    assert_eq!(
        disabled.send("GET", jobs, None, true).await.1["configured"],
        false
    );
    assert_eq!(
        disabled
            .send("POST", jobs, Some(create.clone()), true)
            .await
            .0,
        503
    );
    let mut unconfirmed = create.clone();
    unconfirmed["costConfirmed"] = json!(false);
    assert_eq!(
        operator.send("POST", jobs, Some(unconfirmed), true).await.0,
        400
    );
    let mut wrong = create.clone();
    wrong["token"] = json!("0".repeat(64));
    assert_eq!(operator.send("POST", jobs, Some(wrong), true).await.0, 404);
    let created = operator
        .send("POST", jobs, Some(create.clone()), true)
        .await;
    assert_eq!(created.0, 200);
    assert_eq!(created.1["status"], "submitted");
    let job_path = format!("{jobs}/{}", created.1["id"].as_str().unwrap());
    let mut job = settled(&mut operator, &job_path).await;
    assert_eq!(job["status"], "processing");
    assert_eq!(
        operator
            .send("POST", jobs, Some(create.clone()), true)
            .await
            .0,
        409
    );
    assert_eq!(qwen.calls.lock().unwrap().len(), 1);
    let calls = qwen.calls.lock().unwrap().clone();
    assert_eq!(calls[0]["url"], format!("https://example.test{third_path}"));
    for status in ["DEPLOYING", "mismatch", "UNDEPLOYED", "OK"] {
        *qwen.status.lock().unwrap() = status.into();
        let request = json!({"expectedVersion":job["version"],"voiceId":null,"reason":"query fixed enrollment"});
        let check_path = format!("{job_path}/check");
        assert_eq!(
            operator
                .send("POST", &check_path, Some(request.clone()), true)
                .await
                .0,
            200
        );
        job = settled(&mut operator, &job_path).await;
        assert_eq!(
            job["status"],
            match status {
                "DEPLOYING" => "processing",
                "mismatch" => "modelMismatch",
                "UNDEPLOYED" => "unavailable",
                _ => "ready",
            }
        );
        assert_eq!(
            operator
                .send("POST", &check_path, Some(request), true)
                .await
                .0,
            409
        );
    }
    let listed = operator.send("GET", jobs, None, true).await;
    assert_eq!(listed.1["configured"], true);
    assert!(!listed.1.to_string().contains(third_path));
    assert!(
        !listed
            .1
            .to_string()
            .contains(create["token"].as_str().unwrap())
    );
    assert!(!listed.1.to_string().contains("resource_link"));
    assert_eq!(visitor.send("GET", &job_path, None, true).await.0, 401);
    assert_eq!(learner.send("GET", &job_path, None, true).await.0, 403);
    // Ambiguous creation persists without a retry. Explicit recovery only queries the job's unique prefix.
    let recovery_id = "a".repeat(32);
    let recovery_token = "f".repeat(64);
    db.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"INSERT INTO voice_reference_grants SELECT $1,$2,character_id,character_revision,voice_revision,asset_id,asset_revision,descriptor,reference,actor_id,reason,model,single_speaker_confirmed,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP+interval '15 minutes' FROM voice_reference_grants WHERE id=$3",vec![recovery_id.clone().into(),format!("{:x}",sha2::Sha256::digest(recovery_token.as_bytes())).into(),third.1["grant"]["id"].as_str().unwrap().into()])).await.unwrap();
    qwen.unknown
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let lost=operator.send("POST",jobs,Some(json!({"grantId":recovery_id,"token":recovery_token,"costConfirmed":true,"reason":"unknown test"})),true).await;
    assert_eq!(lost.0, 200);
    let lost_path = format!("{jobs}/{}", lost.1["id"].as_str().unwrap());
    let lost = settled(&mut operator, &lost_path).await;
    assert_eq!(lost["status"], "unknown");
    let calls_before = qwen.calls.lock().unwrap().len();
    assert_eq!(operator.send("POST",&format!("{lost_path}/check"),Some(json!({"expectedVersion":lost["version"],"voiceId":"wrong-prefix","reason":"recover"})),true).await.0,400);
    let recovery_voice = format!(
        "{}-{}-found",
        brioche_server::qwen::MODEL,
        lost["prefix"].as_str().unwrap()
    );
    assert_eq!(operator.send("POST",&format!("{lost_path}/check"),Some(json!({"expectedVersion":lost["version"],"voiceId":recovery_voice,"reason":"recover"})),true).await.0,200);
    assert_eq!(settled(&mut operator, &lost_path).await["status"], "ready");
    assert_eq!(qwen.calls.lock().unwrap().len(), calls_before + 1);
    assert!(
        qwen.calls
            .lock()
            .unwrap()
            .last()
            .unwrap()
            .get("query")
            .is_some()
    );
    assert!(
        db.execute_unprepared("UPDATE voice_clone_jobs SET reason='overwrite'")
            .await
            .is_err()
    );
    let history = operator
        .send("GET", "/api/v1/operator/history", None, true)
        .await
        .1;
    assert!(history.to_string().contains("voiceJobCreated"));
    assert!(history.to_string().contains("voiceJobCheck"));
    assert!(!history.to_string().contains(third_path));
    // Auditions use client attempt IDs, never replay a paid call, and reviews atomically append voices.
    // Earlier reference-length checks intentionally appended Camille v2. Use a separate fixed role
    // for successful acceptance; applying an older candidate over a newer profile must remain rejected.
    let ref_profile = operator
        .send(
            "GET",
            "/api/v1/operator/characters/character-camille/1/voices/1",
            None,
            true,
        )
        .await
        .1["profile"]
        .clone();
    assert_eq!(operator.send("POST","/api/v1/operator/characters",Some(json!({"characterId":"character-luc","characterRevision":1,"expectedVoiceRevision":0,"profile":ref_profile,"reason":"isolated audition source"})),true).await.0,200);
    let source_id = "f".repeat(32);
    db.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"INSERT INTO voice_reference_grants SELECT $1,$2,'character-luc',1,1,asset_id,asset_revision,descriptor,reference,actor_id,'isolated audition source',model,single_speaker_confirmed,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP+interval '15 minutes' FROM voice_reference_grants WHERE id=$3",vec![source_id.clone().into(),"9".repeat(64).into(),third.1["grant"]["id"].as_str().unwrap().into()])).await.unwrap();
    db.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"INSERT INTO voice_clone_jobs(id,grant_id,prefix,actor_id,reason) VALUES($1,$1,'auditionqa',$2,'isolated ready source')",vec![source_id.clone().into(),actor.into()])).await.unwrap();
    db.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"INSERT INTO voice_clone_events(job_id,version,status,voice_id,reason) VALUES($1,1,'ready',$2,'isolated ready source')",vec![source_id.clone().into(),format!("{}-auditionqa-test",brioche_server::qwen::MODEL).into()])).await.unwrap();
    let job = operator
        .send("GET", &format!("{jobs}/{source_id}"), None, true)
        .await
        .1;
    qwen.unknown
        .store(false, std::sync::atomic::Ordering::SeqCst);
    let audition_api = "/api/v1/operator/voice-auditions";
    let audition_id = "b".repeat(32);
    let audition_path = format!("{audition_api}/{audition_id}");
    let audition_request = json!({"id":audition_id,"cloneJobId":job["id"],"expectedCloneVersion":job["version"],"text":"Bonjour !","emotion":"Warm greeting.","costConfirmed":true,"reason":"isolated audition"});
    assert_eq!(
        visitor
            .send("POST", audition_api, Some(audition_request.clone()), true)
            .await
            .0,
        401
    );
    assert_eq!(
        learner
            .send("POST", audition_api, Some(audition_request.clone()), true)
            .await
            .0,
        403
    );
    assert_eq!(
        operator
            .send("POST", audition_api, Some(audition_request.clone()), false)
            .await
            .0,
        403
    );
    let mut bad = audition_request.clone();
    bad["costConfirmed"] = json!(false);
    assert_eq!(
        operator.send("POST", audition_api, Some(bad), true).await.0,
        400
    );
    let mut bad = audition_request.clone();
    bad["expectedCloneVersion"] = json!(99999);
    assert_eq!(
        operator.send("POST", audition_api, Some(bad), true).await.0,
        409
    );
    let mut twin = Browser {
        app: operator.app.clone(),
        cookie: operator.cookie.clone(),
        csrf: operator.csrf.clone(),
    };
    let (first, second) = tokio::join!(
        operator.send("POST", audition_api, Some(audition_request.clone()), true),
        twin.send("POST", audition_api, Some(audition_request.clone()), true)
    );
    assert_eq!(first.0, 200);
    assert_eq!(second.0, 200);
    assert_eq!(first.1["id"], second.1["id"]);
    let audition = settled(&mut operator, &audition_path).await;
    assert_eq!(audition["status"], "ready");
    assert_eq!(audition["durationMs"], 100);
    let synth_count = || {
        qwen.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|v| v.get("synthesis").is_some())
            .count()
    };
    assert_eq!(synth_count(), 1);
    assert_eq!(
        operator
            .send("POST", audition_api, Some(audition_request.clone()), true)
            .await
            .0,
        200
    );
    assert_eq!(synth_count(), 1);
    let mut changed = audition_request.clone();
    changed["emotion"] = json!("Changed request");
    assert_eq!(
        operator
            .send("POST", audition_api, Some(changed), true)
            .await
            .0,
        409
    );
    let file_path = format!("{audition_path}/file");
    assert_eq!(visitor.send("GET", &file_path, None, true).await.0, 401);
    assert_eq!(learner.send("GET", &file_path, None, true).await.0, 403);
    let req = Request::builder()
        .uri(&file_path)
        .header("cookie", &operator.cookie)
        .header("range", "bytes=0-15")
        .body(Body::empty())
        .unwrap();
    let response = operator.app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), 206);
    assert_eq!(response.headers()["cache-control"], "private, no-store");
    assert_eq!(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .len(),
        16
    );
    let review_api = format!("{audition_path}/review");
    let review_request = json!({"accepted":true,"heard":false,"expectedVoiceRevision":1,"reason":"isolated audition acceptance"});
    assert_eq!(
        operator
            .send("POST", &review_api, Some(review_request.clone()), true)
            .await
            .0,
        400
    );
    let mut accepted = review_request.clone();
    accepted["heard"] = json!(true);
    let reviewed = operator
        .send("POST", &review_api, Some(accepted.clone()), true)
        .await;
    assert_eq!(reviewed.0, 200);
    assert_eq!(reviewed.1["appliedVoiceRevision"], 2);
    assert_eq!(reviewed.1["accepted"], true);
    assert_eq!(
        operator
            .send("POST", &review_api, Some(accepted), true)
            .await
            .0,
        409
    );
    let profile = operator
        .send(
            "GET",
            "/api/v1/operator/characters/character-luc/1/voices/2",
            None,
            true,
        )
        .await;
    assert_eq!(profile.0, 200);
    assert_eq!(profile.1["profile"]["voiceId"], job["voiceId"]);
    let mut another = audition_request.clone();
    another["id"] = json!("c".repeat(32));
    assert_eq!(
        operator
            .send("POST", audition_api, Some(another), true)
            .await
            .0,
        200
    );
    let second_path = format!("{audition_api}/{}", "c".repeat(32));
    assert_eq!(
        settled(&mut operator, &second_path).await["status"],
        "ready"
    );
    let rejection = json!({"accepted":false,"heard":true,"expectedVoiceRevision":1,"reason":"isolated audition rejection"});
    assert_eq!(
        operator
            .send(
                "POST",
                &format!("{second_path}/review"),
                Some(rejection),
                true
            )
            .await
            .0,
        200
    );
    let mut third_audition = audition_request.clone();
    third_audition["id"] = json!("d".repeat(32));
    assert_eq!(
        operator
            .send("POST", audition_api, Some(third_audition), true)
            .await
            .0,
        200
    );
    let third_audition_path = format!("{audition_api}/{}", "d".repeat(32));
    assert_eq!(
        settled(&mut operator, &third_audition_path).await["status"],
        "ready"
    );
    let conflict = json!({"accepted":true,"heard":true,"expectedVoiceRevision":1,"reason":"must preserve newer voice"});
    assert_eq!(
        operator
            .send(
                "POST",
                &format!("{third_audition_path}/review"),
                Some(conflict),
                true
            )
            .await
            .0,
        409
    );
    qwen.unknown
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let mut unknown = audition_request;
    unknown["id"] = json!("e".repeat(32));
    assert_eq!(
        operator
            .send("POST", audition_api, Some(unknown), true)
            .await
            .0,
        200
    );
    let unknown_path = format!("{audition_api}/{}", "e".repeat(32));
    assert_eq!(
        settled(&mut operator, &unknown_path).await["status"],
        "unknown"
    );
    assert_eq!(
        operator
            .send("GET", &format!("{unknown_path}/file"), None, true)
            .await
            .0,
        404
    );
    let private_list = operator.send("GET", audition_api, None, true).await;
    assert_eq!(private_list.0, 200);
    assert_eq!(private_list.1["items"].as_array().unwrap().len(), 4);
    assert!(!private_list.1.to_string().contains("Signature"));
    assert!(!private_list.1.to_string().contains("sha256"));
    assert_eq!(
        disabled.send("GET", audition_api, None, true).await.1["configured"],
        false
    );
    let missing_config = json!({"id":"8".repeat(32),"cloneJobId":job["id"],"expectedCloneVersion":job["version"],"text":"Bonjour !","emotion":"Warm greeting.","costConfirmed":true,"reason":"no configured synthesis"});
    assert_eq!(
        disabled
            .send("POST", audition_api, Some(missing_config), true)
            .await
            .0,
        503
    );
    // Actual media integrity and bounded listings are independent of successful provider receipts.
    let audio_sha_row=db.query_one_raw(Statement::from_sql_and_values(DbBackend::Postgres,"SELECT result->>'sha256' AS sha FROM voice_audition_events WHERE audition_id=$1 AND status='ready'",vec![audition_id.clone().into()])).await.unwrap().unwrap();
    let audio_sha: String = audio_sha_row.try_get("", "sha").unwrap();
    let stored = root.join(format!("{audio_sha}.wav"));
    let intact = std::fs::read(&stored).unwrap();
    std::fs::write(&stored, b"damaged").unwrap();
    assert_eq!(operator.send("GET", &file_path, None, true).await.0, 503);
    std::fs::write(&stored, &intact).unwrap();
    let h = operator
        .send("GET", "/api/v1/operator/history", None, true)
        .await
        .1;
    assert!(h.to_string().contains("voiceAuditionAccepted"));
    assert!(h.to_string().contains("voiceAuditionRejected"));
    for i in 0..25u32 {
        let id = format!("{i:032x}");
        db.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"INSERT INTO voice_auditions SELECT $1,clone_job_id,clone_version,profile,parameters,actor_id,reason,CURRENT_TIMESTAMP FROM voice_auditions WHERE id=$2",vec![id.clone().into(),audition_id.clone().into()])).await.unwrap();
        db.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"INSERT INTO voice_audition_events(audition_id,version,status,created_at) VALUES($1,1,'submitted',CURRENT_TIMESTAMP-interval '301 seconds')",vec![id.into()])).await.unwrap();
    }
    let first_auditions = operator.send("GET", audition_api, None, true).await.1;
    assert_eq!(first_auditions["items"].as_array().unwrap().len(), 20);
    assert_eq!(first_auditions["items"][0]["status"], "unknown");
    let last_auditions = operator
        .send(
            "GET",
            &format!(
                "{audition_api}?afterId={}",
                first_auditions["next"].as_str().unwrap()
            ),
            None,
            true,
        )
        .await
        .1;
    assert_eq!(last_auditions["items"].as_array().unwrap().len(), 9);
    assert_eq!(
        operator
            .send(
                "GET",
                &format!("{audition_api}?cloneJobId={source_id}"),
                None,
                true
            )
            .await
            .1["items"]
            .as_array()
            .unwrap()
            .len(),
        20
    );
    assert_eq!(
        operator
            .send(
                "GET",
                &format!("{audition_api}?afterId=invalid"),
                None,
                true
            )
            .await
            .0,
        400
    );
    assert!(
        db.execute_unprepared("UPDATE voice_auditions SET reason='overwrite'")
            .await
            .is_err()
    );
    assert!(
        db.execute_unprepared("DELETE FROM voice_audition_events")
            .await
            .is_err()
    );
    assert!(
        db.execute_unprepared("DELETE FROM voice_audition_reviews")
            .await
            .is_err()
    );
    // Every job remains reachable with a bounded cursor; abandoned workers become unknown, never resent.
    for i in 0..25u32 {
        let id = format!("{i:032x}");
        db.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"INSERT INTO voice_clone_jobs(id,grant_id,prefix,actor_id,reason) VALUES($1,$1,$2,$3,'pagination fixture')",vec![id.clone().into(),format!("t{i}").into(),actor.into()])).await.unwrap();
        db.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"INSERT INTO voice_clone_events(job_id,version,status,reason,created_at) VALUES($1,1,'submitted','abandoned fixture',CURRENT_TIMESTAMP-interval '61 seconds')",vec![id.into()])).await.unwrap();
    }
    let first_jobs = operator.send("GET", jobs, None, true).await.1;
    assert_eq!(first_jobs["items"].as_array().unwrap().len(), 20);
    assert_eq!(first_jobs["items"][0]["status"], "unknown");
    let next_jobs = operator
        .send(
            "GET",
            &format!("{jobs}?afterId={}", first_jobs["next"].as_str().unwrap()),
            None,
            true,
        )
        .await
        .1;
    assert_eq!(next_jobs["items"].as_array().unwrap().len(), 8);
    assert_eq!(
        operator
            .send("GET", &format!("{jobs}?afterId=invalid"), None, true)
            .await
            .0,
        400
    );
    assert!(
        db.execute_unprepared("DELETE FROM voice_clone_events")
            .await
            .is_err()
    );
    assert!(
        brioche_migration::Migrator::down(&db, Some(1))
            .await
            .is_err()
    );
    db.close().await.unwrap();
    admin
        .execute_unprepared(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    std::fs::remove_dir_all(&root).unwrap();
}
#[tokio::test]
#[ignore = "set TEST_DATABASE_URL to a dedicated PostgreSQL database"]
async fn pending_links_are_private_revocable_and_serialized_with_consumption() {
    let url = std::env::var("TEST_DATABASE_URL").unwrap();
    let admin = Database::connect(&url).await.unwrap();
    let schema = format!(
        "token_admin_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    admin
        .execute_unprepared(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let mut options = ConnectOptions::new(url);
    options.set_schema_search_path(&schema);
    let db = Database::connect(options).await.unwrap();
    brioche_migration::Migrator::up(&db, None).await.unwrap();
    let backend = Backend::new(db.clone()).await.unwrap();
    let app = identity::router_with_media_root(
        backend.clone(),
        CsrfPolicy::new(["http://localhost:5173".into()]).unwrap(),
        false,
        std::env::temp_dir(),
    );
    let mut visitor = Browser::new(app.clone()).await;
    let mut operator = Browser::new(app.clone()).await;
    operator
        .register(&backend, "operator@example.test", true)
        .await;
    let mut learner = Browser::new(app).await;
    learner
        .register(&backend, "learner@example.test", false)
        .await;
    let path = "/api/v1/operator/accounts/pending-tokens";
    assert_eq!(visitor.send("GET", path, None, true).await.0, 401);
    assert_eq!(learner.send("GET", path, None, true).await.0, 403);
    let invite = backend
        .issue_token("pending@example.test", false, false)
        .await
        .unwrap();
    let reset_token = backend
        .issue_token("learner@example.test", true, false)
        .await
        .unwrap();
    db.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"INSERT INTO identity_tokens(token_hash,kind,email,expires_at) VALUES($1,'invite','expired@example.test',CURRENT_TIMESTAMP-interval '1 second')",vec!["e".repeat(64).into()])).await.unwrap();
    let listed = operator.send("GET", path, None, true).await;
    assert_eq!(listed.0, 200);
    assert_eq!(listed.1["items"].as_array().unwrap().len(), 2);
    assert!(!listed.1.to_string().contains(&invite));
    assert!(!listed.1.to_string().contains("tokenHash"));
    let id = listed.1["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["email"] == "pending@example.test")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let revoke = format!("{path}/{id}/revoke");
    assert_eq!(
        learner
            .send("POST", &revoke, Some(json!({"reason":"isolated"})), true)
            .await
            .0,
        403
    );
    assert_eq!(
        operator
            .send("POST", &revoke, Some(json!({"reason":"isolated"})), false)
            .await
            .0,
        403
    );
    assert_eq!(
        operator
            .send("POST", &revoke, Some(json!({"reason":"isolated"})), true)
            .await
            .0,
        200
    );
    assert_eq!(
        operator
            .send("POST", &revoke, Some(json!({"reason":"isolated"})), true)
            .await
            .0,
        404
    );
    assert!(
        backend
            .accept_invite(brioche_course_contract::AcceptInviteRequest {
                token: invite,
                email: "pending@example.test".into(),
                display_name: "Pending".into(),
                password: "correct horse brioche fromage".into()
            })
            .await
            .is_err()
    );
    let remaining = operator
        .send("GET", &format!("{path}?kind=invite"), None, true)
        .await;
    assert_eq!(remaining.1["items"].as_array().unwrap().len(), 0);
    assert_eq!(
        operator
            .send("GET", &format!("{path}?kind=unknown"), None, true)
            .await
            .0,
        400
    );
    let reset_list = operator
        .send("GET", &format!("{path}?kind=reset"), None, true)
        .await;
    let reset_id = reset_list.1["items"][0]["id"].as_str().unwrap();
    assert_eq!(
        operator
            .send(
                "POST",
                &format!("{path}/{reset_id}/revoke"),
                Some(json!({"reason":"isolated reset revocation"})),
                true
            )
            .await
            .0,
        200
    );
    assert!(
        backend
            .reset_password(brioche_course_contract::ResetPasswordRequest {
                token: reset_token,
                password: "changed but rejected brioche password".into()
            })
            .await
            .is_err()
    );
    assert_eq!(learner.send("GET", "/api/v1/me", None, true).await.0, 200);
    assert_eq!(
        operator
            .send("GET", &format!("{path}?afterId=bad"), None, true)
            .await
            .0,
        400
    );
    let audited=db.query_one_raw(Statement::from_string(DbBackend::Postgres,"SELECT count(*) AS n FROM account_admin_audit a JOIN users u ON u.id=a.actor_id WHERE a.action IN ('revokeInvite','revokeReset') AND u.email='operator@example.test'".to_owned())).await.unwrap().unwrap().try_get::<i64>("","n").unwrap();
    assert_eq!(audited, 2);
    let history = operator
        .send("GET", "/api/v1/operator/history", None, true)
        .await;
    assert!(
        history.1["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["action"] == "revokeInvite")
    );
    assert!(
        history.1["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["action"] == "revokeReset")
    );
    for i in 0..25 {
        db.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"INSERT INTO identity_tokens(token_hash,kind,email,role,expires_at) VALUES($1,'invite',$2,'learner',CURRENT_TIMESTAMP+interval '1 day')",vec![format!("{i:064x}").into(),format!("synthetic-{i}@example.test").into()])).await.unwrap();
    }
    let page = operator
        .send("GET", &format!("{path}?kind=invite"), None, true)
        .await;
    assert_eq!(page.1["items"].as_array().unwrap().len(), 20);
    let next = operator
        .send(
            "GET",
            &format!(
                "{path}?kind=invite&afterId={}",
                page.1["nextId"].as_str().unwrap()
            ),
            None,
            true,
        )
        .await;
    assert_eq!(next.1["items"].as_array().unwrap().len(), 5);
    assert_eq!(learner.send("GET", "/api/v1/me", None, true).await.0, 200);
    let race = backend
        .issue_token("race@example.test", false, false)
        .await
        .unwrap();
    // Query the synthetic identifier directly if the first page is filled by other records.
    use sha2::{Digest, Sha256};
    let record_id = format!(
        "{:x}",
        Sha256::digest(format!("{:x}", Sha256::digest(&race)))
    );
    let race_path = format!("{path}/{record_id}/revoke");
    let (revoked, accepted) = tokio::join!(
        operator.send(
            "POST",
            &race_path,
            Some(json!({"reason":"race test"})),
            true
        ),
        backend.accept_invite(brioche_course_contract::AcceptInviteRequest {
            token: race,
            email: "race@example.test".into(),
            display_name: "Race".into(),
            password: "correct horse brioche fromage".into()
        })
    );
    assert_ne!(revoked.0 == 200, accepted.is_ok());
    db.close().await.unwrap();
    admin
        .execute_unprepared(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
}
impl Browser {
    async fn new(app: Router) -> Self {
        let mut result = Self {
            app,
            cookie: String::new(),
            csrf: String::new(),
        };
        assert_eq!(
            result.send("GET", "/api/v1/auth/csrf", None, true).await.0,
            200
        );
        result
    }
    async fn send(
        &mut self,
        method: &str,
        path: &str,
        body: Option<Value>,
        protect: bool,
    ) -> (u16, Value) {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header("cookie", &self.cookie);
        if protect {
            request = request
                .header("origin", "http://localhost:5173")
                .header("x-csrf-token", &self.csrf);
        }
        let request = request
            .header("content-type", "application/json")
            .body(Body::from(
                body.map(|v| serde_json::to_vec(&v).unwrap())
                    .unwrap_or_default(),
            ))
            .unwrap();
        let response = self.app.clone().oneshot(request).await.unwrap();
        let status = response.status().as_u16();
        assert_eq!(response.headers()["cache-control"], "private, no-store");
        for cookie in response.headers().get_all("set-cookie") {
            self.cookie = cookie.to_str().unwrap().split(';').next().unwrap().into();
        }
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let value: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        if let Some(csrf) = value["csrfToken"].as_str() {
            self.csrf = csrf.into();
        }
        (status, value)
    }
    async fn register(&mut self, backend: &Backend, email: &str, operator: bool) {
        let token = backend.issue_token(email, false, operator).await.unwrap();
        assert_eq!(self.send("POST","/api/v1/auth/accept-invite",Some(json!({"token":token,"email":email,"displayName":"Test","password":"correct horse brioche fromage"})),true).await.0,200);
    }
    async fn upload_asset(&self, document: Value, file: &[u8], protect: bool) -> (u16, Value) {
        self.upload_media("/api/v1/operator/assets", document, file, protect)
            .await
    }
    async fn upload_media(
        &self,
        path: &str,
        document: Value,
        file: &[u8],
        protect: bool,
    ) -> (u16, Value) {
        let boundary = "brioche-test-boundary";
        let mut body=format!("--{boundary}\r\nContent-Disposition: form-data; name=\"document\"\r\n\r\n{}\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"../../untrusted.svg\"\r\nContent-Type: image/svg+xml\r\n\r\n",document).into_bytes();
        body.extend_from_slice(file);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
        let mut request = Request::builder()
            .method("POST")
            .uri(path)
            .header("cookie", &self.cookie)
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            );
        if protect {
            request = request
                .header("origin", "http://localhost:5173")
                .header("x-csrf-token", &self.csrf);
        }
        let response = self
            .app
            .clone()
            .oneshot(request.body(Body::from(body)).unwrap())
            .await
            .unwrap();
        let status = response.status().as_u16();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }
}
#[tokio::test]
#[ignore = "set TEST_DATABASE_URL to a dedicated PostgreSQL database"]
async fn approvals_permissions_concurrency_and_publication() {
    let url = std::env::var("TEST_DATABASE_URL").unwrap();
    let admin = Database::connect(&url).await.unwrap();
    let schema = format!(
        "admin_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    admin
        .execute_unprepared(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let mut options = ConnectOptions::new(url);
    options.set_schema_search_path(&schema);
    let db = Database::connect(options).await.unwrap();
    brioche_migration::Migrator::up(&db, None).await.unwrap();
    let root = assets::fixture_assets(&db, &schema).await;
    let backend = Backend::new(db.clone()).await.unwrap();
    let app = identity::router_with_media_root(
        backend.clone(),
        CsrfPolicy::new(["http://localhost:5173".into()]).unwrap(),
        false,
        root.clone(),
    );
    let mut visitor = Browser::new(app.clone()).await;
    assert_eq!(
        visitor
            .send("GET", "/api/v1/operator/overview", None, true)
            .await
            .0,
        401
    );
    let mut learner = Browser::new(app.clone()).await;
    learner
        .register(&backend, "learner@example.test", false)
        .await;
    assert_eq!(
        learner
            .send("GET", "/api/v1/operator/overview", None, true)
            .await
            .0,
        403
    );
    let mut operator = Browser::new(app.clone()).await;
    operator
        .register(&backend, "operator@example.test", true)
        .await;
    let voices_path = "/api/v1/operator/characters";
    let revisions_path = "/api/v1/operator/characters/revisions";
    let character_request = json!({"characterId":"character-qa","expectedRevision":0,"displayName":"Test original","avatarId":"avatar-camille-v1","avatarRevision":1,"reason":"isolated character creation"});
    assert_eq!(
        visitor
            .send(
                "POST",
                revisions_path,
                Some(character_request.clone()),
                true
            )
            .await
            .0,
        401
    );
    assert_eq!(
        learner
            .send(
                "POST",
                revisions_path,
                Some(character_request.clone()),
                true
            )
            .await
            .0,
        403
    );
    assert_eq!(
        operator
            .send(
                "POST",
                revisions_path,
                Some(character_request.clone()),
                false
            )
            .await
            .0,
        403
    );
    let created = operator
        .send(
            "POST",
            revisions_path,
            Some(character_request.clone()),
            true,
        )
        .await;
    assert_eq!(created.0, 200);
    assert_eq!(created.1["character"]["revision"], 1);
    assert_eq!(created.1["voiceRevision"], 0);
    assert!(created.1["profile"].is_null());
    assert_eq!(
        operator
            .send(
                "POST",
                revisions_path,
                Some(character_request.clone()),
                true
            )
            .await
            .0,
        409
    );
    let mut update = character_request.clone();
    update["expectedRevision"] = json!(1);
    update["displayName"] = json!("Test new");
    update["avatarId"] = json!("avatar-luc-v1");
    let mut peer = Browser {
        app: operator.app.clone(),
        cookie: operator.cookie.clone(),
        csrf: operator.csrf.clone(),
    };
    let (left, right) = tokio::join!(
        operator.send("POST", revisions_path, Some(update.clone()), true),
        peer.send("POST", revisions_path, Some(update.clone()), true)
    );
    let mut statuses = vec![left.0, right.0];
    statuses.sort();
    assert_eq!(statuses, vec![200, 409]);
    for path in [
        "/api/v1/operator/characters/character-qa/1",
        "/api/v1/operator/characters/character-qa/2",
    ] {
        assert_eq!(visitor.send("GET", path, None, true).await.0, 401);
        assert_eq!(learner.send("GET", path, None, true).await.0, 403);
    }
    let old = operator
        .send(
            "GET",
            "/api/v1/operator/characters/character-qa/1",
            None,
            true,
        )
        .await;
    assert_eq!(old.0, 200);
    assert_eq!(old.1["character"]["displayName"], "Test original");
    assert_eq!(old.1["character"]["avatarId"], "avatar-camille-v1");
    let latest = operator
        .send(
            "GET",
            "/api/v1/operator/characters/character-qa/2",
            None,
            true,
        )
        .await;
    assert_eq!(latest.1["character"]["displayName"], "Test new");
    assert!(latest.1["profile"].is_null());
    for (key, value) in [
        ("avatarId", json!("art-bakery-morning")),
        ("avatarId", json!("unregistered-avatar")),
        ("displayName", json!("")),
        ("reason", json!("")),
    ] {
        let mut invalid = update.clone();
        invalid["expectedRevision"] = json!(2);
        invalid[key] = value;
        assert_eq!(
            operator
                .send("POST", revisions_path, Some(invalid), true)
                .await
                .0,
            400
        );
    }
    assert!(
        db.execute_unprepared(
            "UPDATE character_revisions SET snapshot='{}' WHERE character_id='character-qa'"
        )
        .await
        .is_err()
    );
    let history = operator
        .send("GET", "/api/v1/operator/history", None, true)
        .await;
    assert!(
        history.1["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["target"] == "character-qa v2" && i["action"] == "assetImport")
    );
    let upload = json!({"assetId":"qa-web-upload","revision":1,"mimeType":"image/svg+xml","altZh":"隔离上传","creditZh":"仅测试","source":"test:synthetic","license":"LicenseRef-TestOnly","creator":"test fixture","rightsConfirmed":true,"reason":"isolated asset upload"});
    let audio_upload = json!({"assetId":"qa-web-recording","revision":1,"mimeType":"audio/mpeg","creditZh":"仅测试","source":"test:synthetic","license":"LicenseRef-TestOnly","creator":"test fixture","rightsConfirmed":true,"reason":"isolated recording upload"});
    let recording = include_bytes!("fixtures/audio/synthetic.mp3");
    let audio_path = "/api/v1/operator/recordings";
    assert_eq!(
        visitor
            .upload_media(audio_path, audio_upload.clone(), recording, true)
            .await
            .0,
        401
    );
    assert_eq!(
        learner
            .upload_media(audio_path, audio_upload.clone(), recording, true)
            .await
            .0,
        403
    );
    assert_eq!(
        operator
            .upload_media(audio_path, audio_upload.clone(), recording, false)
            .await
            .0,
        403
    );
    for (field, value) in [
        ("rightsConfirmed", json!(false)),
        ("assetId", json!("../escape")),
        ("reason", json!("")),
        ("mimeType", json!("audio/ogg")),
    ] {
        let mut bad = audio_upload.clone();
        bad[field] = value;
        assert_eq!(
            operator
                .upload_media(audio_path, bad, recording, true)
                .await
                .0,
            400
        );
    }
    assert_eq!(
        operator
            .upload_media(audio_path, audio_upload.clone(), b"not audio", true)
            .await
            .0,
        400
    );
    assert_eq!(
        operator
            .upload_media(
                audio_path,
                audio_upload.clone(),
                &recording[..recording.len() - 1],
                true
            )
            .await
            .0,
        400
    );
    let uploaded = operator
        .upload_media(audio_path, audio_upload.clone(), recording, true)
        .await;
    assert_eq!(uploaded.0, 200);
    assert_eq!(
        uploaded.1,
        json!({"assetId":"qa-web-recording","revision":1})
    );
    assert_eq!(
        operator
            .upload_media(audio_path, audio_upload.clone(), recording, true)
            .await
            .0,
        409
    );
    let row=db.query_one_raw(Statement::from_string(DbBackend::Postgres,"SELECT count(*) AS n FROM audio_import_audit a JOIN users u ON a.actor_id=u.id WHERE u.email='operator@example.test' AND a.reason='isolated recording upload'".to_owned())).await.unwrap().unwrap();
    assert_eq!(row.try_get::<i64>("", "n").unwrap(), 1);
    let audio_history = operator
        .send("GET", "/api/v1/operator/history", None, true)
        .await;
    assert!(
        audio_history.1["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["action"] == "audioImport"
                && i["target"] == "qa-web-recording v1"
                && i["reason"] == "isolated recording upload")
    );
    let listed = operator.send("GET", audio_path, None, true).await;
    assert_eq!(listed.1["items"][0]["asset"]["durationMs"], 1000);
    assert_eq!(listed.1["items"][0]["sampleRate"], 24000);
    let mut concurrent = audio_upload.clone();
    concurrent["assetId"] = json!("qa-concurrent-audio");
    concurrent["reason"] = json!("concurrent recording");
    let (left, right) = tokio::join!(
        operator.upload_media(audio_path, concurrent.clone(), recording, true),
        operator.upload_media(audio_path, concurrent, recording, true)
    );
    let mut outcomes = [left.0, right.0];
    outcomes.sort();
    assert_eq!(outcomes, [200, 409]);
    assert!(
        brioche_migration::Migrator::migrations()
            .into_iter()
            .find(|m| m.name() == "m20261007_000017_recording_admin")
            .unwrap()
            .down(&sea_orm_migration::SchemaManager::new(&db))
            .await
            .is_err(),
        "operator recording audit cannot be removed by rollback"
    );
    brioche_migration::Migrator::up(&db, None).await.unwrap();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/operator/recordings/qa-web-recording/1/file")
                .header("cookie", &operator.cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(
        &response.into_body().collect().await.unwrap().to_bytes()[..],
        recording
    );
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 96 96\"><!--{}--><rect width=\"96\" height=\"96\" fill=\"red\"/></svg>",
        "x".repeat(20_000)
    );
    assert_eq!(
        visitor
            .upload_asset(upload.clone(), svg.as_bytes(), true)
            .await
            .0,
        401
    );
    assert_eq!(
        learner
            .upload_asset(upload.clone(), svg.as_bytes(), true)
            .await
            .0,
        403
    );
    assert_eq!(
        operator
            .upload_asset(upload.clone(), svg.as_bytes(), false)
            .await
            .0,
        403
    );
    let mut invalid = upload.clone();
    invalid["rightsConfirmed"] = json!(false);
    assert_eq!(
        operator.upload_asset(invalid, svg.as_bytes(), true).await.0,
        400
    );
    assert_eq!(operator.upload_asset(upload.clone(),b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1\" height=\"1\"><script>alert(1)</script></svg>",true).await.0,400);
    assert_eq!(
        operator
            .upload_asset(upload.clone(), b"not an image", true)
            .await
            .0,
        400
    );
    let first_upload = operator
        .upload_asset(upload.clone(), svg.as_bytes(), true)
        .await;
    assert_eq!(first_upload.0, 200);
    assert_eq!(
        first_upload.1,
        json!({"assetId":"qa-web-upload","revision":1})
    );
    assert_eq!(
        operator
            .upload_asset(upload.clone(), svg.as_bytes(), true)
            .await
            .0,
        409
    );
    let row=db.query_one_raw(Statement::from_string(DbBackend::Postgres,"SELECT count(*) AS n FROM asset_import_audit a JOIN users u ON a.actor_id=u.id WHERE u.email='operator@example.test' AND a.reason='isolated asset upload'".to_owned())).await.unwrap().unwrap();
    assert_eq!(row.try_get::<i64>("", "n").unwrap(), 1);
    let upload_history = operator
        .send("GET", "/api/v1/operator/history", None, true)
        .await;
    assert!(
        upload_history.1["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["action"] == "assetImport"
                && item["target"] == "qa-web-upload v1"
                && item["reason"] == "isolated asset upload")
    );
    let mut invalid = upload.clone();
    invalid["assetId"] = json!("../escape");
    assert_eq!(
        operator.upload_asset(invalid, svg.as_bytes(), true).await.0,
        400
    );
    let mut invalid = upload.clone();
    invalid["reason"] = json!("");
    assert_eq!(
        operator.upload_asset(invalid, svg.as_bytes(), true).await.0,
        400
    );
    let image = operator
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/operator/assets/qa-web-upload/1/file")
                .header("cookie", &operator.cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(image.status(), 200);
    assert_eq!(
        image
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .as_ref(),
        svg.as_bytes()
    );
    let assets_path = "/api/v1/operator/assets";
    let file_path = "/api/v1/operator/assets/avatar-camille-v1/1/file";
    for path in [assets_path, file_path] {
        assert_eq!(visitor.send("GET", path, None, true).await.0, 401);
        assert_eq!(learner.send("GET", path, None, true).await.0, 403);
    }
    let registry = operator.send("GET", assets_path, None, true).await;
    assert_eq!(registry.0, 200);
    assert_eq!(registry.1["items"].as_array().unwrap().len(), 5);
    assert!(registry.1["items"].as_array().unwrap().iter().all(|item| {
        item.get("file").is_none()
            && item.get("provenance").is_none()
            && item["asset"]["url"]
                .as_str()
                .unwrap()
                .starts_with("/api/v1/operator/assets/")
    }));
    assert_eq!(
        operator
            .send(
                "GET",
                "/api/v1/operator/assets?afterId=avatar-camille-v1",
                None,
                true
            )
            .await
            .0,
        400
    );
    assert_eq!(
        operator
            .send(
                "GET",
                "/api/v1/operator/assets?afterId=avatar-camille-v1&afterRevision=0",
                None,
                true
            )
            .await
            .0,
        400
    );
    assert_eq!(
        operator
            .send("GET", "/api/v1/operator/assets?unknown=yes", None, true)
            .await
            .0,
        400
    );
    assert_eq!(
        operator
            .send("GET", "/api/v1/operator/assets?afterRevision=1", None, true)
            .await
            .0,
        400
    );
    // Twenty-five versions of the same ID prove a scalar ID cursor cannot work here.
    db.execute_unprepared(r#"INSERT INTO media_assets(asset_id,revision,descriptor,provenance,sha256,extension,byte_size)
        SELECT 'qa-asset',n,jsonb_set(jsonb_set(descriptor,'{assetId}','"qa-asset"'),'{revision}',to_jsonb(n)),provenance,sha256,extension,byte_size
        FROM media_assets CROSS JOIN generate_series(1,25) n WHERE asset_id='avatar-camille-v1' AND revision=1"#).await.unwrap();
    let first = operator
        .send("GET", "/api/v1/operator/assets?q=qa-asset", None, true)
        .await;
    assert_eq!(first.1["items"].as_array().unwrap().len(), 20);
    assert_eq!(first.1["next"], json!({"assetId":"qa-asset","revision":20}));
    let second = operator
        .send(
            "GET",
            "/api/v1/operator/assets?q=qa-asset&afterId=qa-asset&afterRevision=20",
            None,
            true,
        )
        .await;
    assert_eq!(second.1["items"].as_array().unwrap().len(), 5);
    assert_eq!(second.1["items"][0]["asset"]["revision"], 21);
    assert!(second.1["next"].is_null());
    assert!(
        operator
            .send("GET", "/api/v1/operator/assets?q=%25", None, true)
            .await
            .1["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        operator
            .send(
                "GET",
                "/api/v1/operator/assets/no-such-asset/1/file",
                None,
                true
            )
            .await
            .0,
        404
    );
    let response = operator
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(file_path)
                .header("cookie", &operator.cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["content-type"], "image/svg+xml");
    assert_eq!(response.headers()["cache-control"], "private, no-store");
    assert_eq!(response.headers()["x-content-type-options"], "nosniff");
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(
        bytes.as_ref(),
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/preview/avatars/camille.svg")
        )
        .unwrap()
    );
    assert_eq!(visitor.send("GET", voices_path, None, true).await.0, 401);
    assert_eq!(learner.send("GET", voices_path, None, true).await.0, 403);
    let listed = operator.send("GET", voices_path, None, true).await;
    assert_eq!(listed.0, 200);
    assert!(
        listed.1["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| v["voiceRevision"] == 0)
    );
    let seed: Value =
        serde_json::from_str(include_str!("../../../docs/characters/voices.json")).unwrap();
    let mut voice_request = json!({"characterId":"character-camille","characterRevision":1,"expectedVoiceRevision":0,"profile":seed["items"][0]["profile"],"reason":"isolated voice profile test"});
    assert_eq!(
        learner
            .send("POST", voices_path, Some(voice_request.clone()), true)
            .await
            .0,
        403
    );
    assert_eq!(
        operator
            .send("POST", voices_path, Some(voice_request.clone()), false)
            .await
            .0,
        403
    );
    assert_eq!(
        operator
            .send("POST", voices_path, Some(voice_request.clone()), true)
            .await
            .0,
        200
    );
    assert_eq!(
        operator
            .send("POST", voices_path, Some(voice_request.clone()), true)
            .await
            .0,
        409
    );
    voice_request["expectedVoiceRevision"] = json!(1);
    voice_request["profile"]["voiceKind"] = json!("cloned");
    assert_eq!(
        operator
            .send("POST", voices_path, Some(voice_request.clone()), true)
            .await
            .0,
        400
    );
    voice_request["profile"]["voiceKind"] = json!("system");
    voice_request["profile"]["referenceAudio"] = json!({"assetId":"qa-web-recording","revision":1,"transcript":"Bonjour !","cloningPermission":"Synthetic protocol fixture; no real speaker"});
    let mut missing = voice_request.clone();
    missing["profile"]["referenceAudio"]["revision"] = json!(2);
    assert_eq!(
        operator
            .send("POST", voices_path, Some(missing), true)
            .await
            .0,
        404
    );
    let mut unauthorized = voice_request.clone();
    unauthorized["profile"]["referenceAudio"]["cloningPermission"] = json!("");
    assert_eq!(
        operator
            .send("POST", voices_path, Some(unauthorized), true)
            .await
            .0,
        400
    );
    db.execute_unprepared(r#"INSERT INTO audio_assets(asset_id,revision,descriptor,provenance,sha256,extension,byte_size,duration_ms,sample_rate,channels) SELECT 'qa-long-reference',1,jsonb_set(jsonb_set(descriptor,'{assetId}','"qa-long-reference"'),'{durationMs}','31000'),provenance,sha256,extension,byte_size,31000,sample_rate,channels FROM audio_assets WHERE asset_id='qa-web-recording' AND revision=1"#).await.unwrap();
    let mut long = voice_request.clone();
    long["profile"]["referenceAudio"]["assetId"] = json!("qa-long-reference");
    assert_eq!(
        operator.send("POST", voices_path, Some(long), true).await.0,
        400
    );
    voice_request["profile"]["defaultEmotion"] = json!("Quiet and calm.");
    assert_eq!(
        operator
            .send("POST", voices_path, Some(voice_request), true)
            .await
            .0,
        200
    );
    let fixed = operator
        .send(
            "GET",
            "/api/v1/operator/characters/character-camille/1/voices/1",
            None,
            true,
        )
        .await;
    assert_eq!(fixed.0, 200);
    assert_eq!(
        fixed.1["profile"]["defaultEmotion"],
        seed["items"][0]["profile"]["defaultEmotion"]
    );
    assert!(
        db.execute_unprepared("UPDATE character_voice_profiles SET reason='mutated'")
            .await
            .is_err()
    );
    let cli_request = root.join("luc-voice.json");
    std::fs::write(&cli_request,json!({"characterId":"character-luc","characterRevision":1,"expectedVoiceRevision":0,"profile":seed["items"][1]["profile"],"reason":"CLI test"}).to_string()).unwrap();
    let separator = if std::env::var("TEST_DATABASE_URL").unwrap().contains('?') {
        "&"
    } else {
        "?"
    };
    let cli_url = format!(
        "{}{separator}options=-csearch_path%3D{schema}",
        std::env::var("TEST_DATABASE_URL").unwrap()
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_brioche-server"))
        .current_dir(&root)
        .env("DATABASE_URL", cli_url)
        .env("CONTENT_MODE", "database")
        .env("API_BIND", "invalid-bind-must-not-be-used")
        .args([
            "character-voice-import",
            cli_request.to_str().unwrap(),
            "operator@example.test",
            "isolated CLI profile initialization",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("registered at revision 1"));
    let mut source: Value =
        serde_json::from_str(include_str!("../../../docs/examples/a1-bakery.lesson.json")).unwrap();
    source["assetRefs"] = assets::fixture_refs();
    source["editorial"] = json!({"status":"draft","note":"synthetic draft"});
    source["summaryZh"] = json!("导入正文边界".repeat(1500));
    let import_request = json!({"document":source.to_string(),"reason":"operator import test"});
    assert!(serde_json::to_vec(&import_request).unwrap().len() > 16 * 1024);
    assert_eq!(
        learner
            .send(
                "POST",
                "/api/v1/operator/lessons/import",
                Some(import_request.clone()),
                true
            )
            .await
            .0,
        403
    );
    assert_eq!(
        operator
            .send(
                "POST",
                "/api/v1/operator/lessons/import",
                Some(import_request.clone()),
                false
            )
            .await
            .0,
        403
    );
    let mut missing_asset = source.clone();
    missing_asset["assetRefs"][0]["assetId"] = json!("unregistered-asset");
    assert_eq!(
        operator
            .send(
                "POST",
                "/api/v1/operator/lessons/import",
                Some(json!({"document":missing_asset.to_string(),"reason":"invalid asset"})),
                true
            )
            .await
            .0,
        400
    );
    assert_eq!(operator.send("POST","/api/v1/operator/lessons/import",Some(json!({"document":"{\"id\":\"a\",\"id\":\"b\"}","reason":"duplicate JSON member"})),true).await.0,400);
    assert_eq!(
        operator
            .send(
                "POST",
                "/api/v1/operator/lessons/import",
                Some(import_request.clone()),
                true
            )
            .await
            .0,
        200
    );
    assert_eq!(
        operator
            .send(
                "POST",
                "/api/v1/operator/lessons/import",
                Some(import_request.clone()),
                true
            )
            .await
            .0,
        200
    );
    let mut different = source.clone();
    different["summaryZh"] = json!("different content");
    assert_eq!(
        operator
            .send(
                "POST",
                "/api/v1/operator/lessons/import",
                Some(json!({"document":different.to_string(),"reason":"immutable conflict"})),
                true
            )
            .await
            .0,
        409
    );
    let imports = db
        .query_one_raw(Statement::from_string(
            DbBackend::Postgres,
            "SELECT count(*) AS n FROM lesson_import_audit",
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap();
    assert_eq!(imports, 1);
    let source = brioche_server::media::hydrate_source(&db, source)
        .await
        .unwrap();
    let lesson = brioche_server::project_source(source.clone()).unwrap();
    let path = format!("/api/v1/operator/lessons/{}/revisions/1/review", lesson.id);
    let decision = json!({"version":0,"approved":true,"reason":"Test approval"});
    assert_eq!(
        learner
            .send("POST", &path, Some(decision.clone()), true)
            .await
            .0,
        403
    );
    assert_eq!(
        operator
            .send("POST", &path, Some(decision.clone()), false)
            .await
            .0,
        403
    );
    let overview = operator
        .send("GET", "/api/v1/operator/overview", None, true)
        .await;
    assert_eq!(overview.0, 200);
    assert_eq!(overview.1["lessons"][0]["approved"], false);
    let encoded = overview.1.to_string();
    assert!(
        !encoded.contains("serverOnly")
            && !encoded.contains("accepted")
            && !encoded.contains("correctOptionId")
    );
    let manifest:brioche_server::content::ReleaseManifest=serde_json::from_value(json!({"id":"admin-test","schemaVersion":"1.0","levels":[{"id":lesson.level_id,"label":"A1","units":[{"id":lesson.unit_id,"titleZh":"test","lessons":[{"lessonId":lesson.id,"revision":1}]}]}]})).unwrap();
    let stage_request = json!({"document":serde_json::to_string(&manifest).unwrap(),"reason":"stage through admin"});
    assert_eq!(
        learner
            .send(
                "POST",
                "/api/v1/operator/releases/stage",
                Some(stage_request.clone()),
                true
            )
            .await
            .0,
        403
    );
    assert_eq!(
        operator
            .send(
                "POST",
                "/api/v1/operator/releases/stage",
                Some(stage_request.clone()),
                true
            )
            .await
            .0,
        400
    );
    assert!(
        brioche_server::content::stage(&db, &manifest, "test", "test", &root)
            .await
            .is_err()
    );
    let approved = operator
        .send("POST", &path, Some(decision.clone()), true)
        .await;
    assert_eq!(approved.0, 200);
    assert_eq!(approved.1["version"], 1);
    assert_eq!(
        operator
            .send("POST", &path, Some(decision.clone()), true)
            .await
            .0,
        200
    );
    assert_eq!(
        operator
            .send(
                "POST",
                &path,
                Some(json!({"version":0,"approved":false,"reason":"stale"})),
                true
            )
            .await
            .0,
        409
    );
    assert_eq!(
        operator
            .send(
                "POST",
                "/api/v1/operator/releases/stage",
                Some(stage_request.clone()),
                true
            )
            .await
            .0,
        200
    );
    assert_eq!(
        operator
            .send(
                "POST",
                "/api/v1/operator/releases/stage",
                Some(stage_request.clone()),
                true
            )
            .await
            .0,
        409
    );
    assert_eq!(
        operator
            .send(
                "POST",
                &path,
                Some(json!({"version":1,"approved":false,"reason":"return staged course"})),
                true
            )
            .await
            .0,
        200
    );
    assert_eq!(
        operator
            .send(
                "POST",
                "/api/v1/operator/releases/activate",
                Some(json!({"releaseId":"admin-test","generation":"0","reason":"test"})),
                true
            )
            .await
            .0,
        409
    );
    assert_eq!(
        operator
            .send(
                "POST",
                &path,
                Some(json!({"version":2,"approved":true,"reason":"approve again"})),
                true
            )
            .await
            .0,
        200
    );
    assert_eq!(
        operator
            .send(
                "POST",
                "/api/v1/operator/releases/activate",
                Some(json!({"releaseId":"admin-test","generation":"0","reason":"test"})),
                true
            )
            .await
            .0,
        200
    );
    assert_eq!(
        operator
            .send(
                "POST",
                &path,
                Some(json!({"version":3,"approved":false,"reason":"published"})),
                true
            )
            .await
            .0,
        409
    );
    let saved = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT server_document FROM lesson_revisions WHERE lesson_id=$1",
            [lesson.id.clone().into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<Value>("", "server_document")
        .unwrap();
    assert_eq!(saved, source);
    let mut second_source = source.clone();
    second_source["revision"] = json!(2);
    let second_lesson = brioche_server::project_source(second_source.clone()).unwrap();
    db.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"INSERT INTO lesson_revisions(lesson_id,revision,published,public_document,server_document) VALUES($1,2,false,$2,$3)",[lesson.id.clone().into(),serde_json::to_value(second_lesson).unwrap().into(),second_source.into()])).await.unwrap();
    let mut second_tab = Browser {
        app: operator.app.clone(),
        cookie: operator.cookie.clone(),
        csrf: operator.csrf.clone(),
    };
    let second_path = format!("/api/v1/operator/lessons/{}/revisions/2/review", lesson.id);
    let (a, b) = tokio::join!(
        operator.send(
            "POST",
            &second_path,
            Some(json!({"version":0,"approved":true,"reason":"concurrent approval"})),
            true
        ),
        second_tab.send(
            "POST",
            &second_path,
            Some(json!({"version":0,"approved":false,"reason":"concurrent rejection"})),
            true
        )
    );
    let mut statuses = [a.0, b.0];
    statuses.sort();
    assert_eq!(statuses, [200, 409]);
    let concurrent_count = db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT count(*) AS n FROM editorial_reviews WHERE lesson_id=$1 AND revision=2",
            [lesson.id.clone().into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap();
    assert_eq!(concurrent_count, 1);
    let withdraw = format!(
        "/api/v1/operator/lessons/{}/revisions/1/withdraw",
        lesson.id
    );
    assert_eq!(
        operator
            .send(
                "POST",
                &withdraw,
                Some(json!({"generation":"0","reason":"stale"})),
                true
            )
            .await
            .0,
        409
    );
    assert_eq!(
        operator
            .send(
                "POST",
                &withdraw,
                Some(json!({"generation":"1","reason":"test withdrawal"})),
                true
            )
            .await
            .0,
        200
    );
    assert_eq!(
        operator
            .send(
                "POST",
                &path,
                Some(json!({"version":3,"approved":true,"reason":"withdrawn"})),
                true
            )
            .await
            .0,
        410
    );
    let latest = operator
        .send("GET", "/api/v1/operator/overview", None, true)
        .await;
    assert_eq!(latest.1["generation"], "2");
    for (browser, status) in [(&mut visitor, 401), (&mut learner, 403)] {
        assert_eq!(
            browser
                .send("GET", "/api/v1/operator/history", None, true)
                .await
                .0,
            status
        );
    }
    assert_eq!(
        operator
            .send("GET", "/api/v1/operator/history?beforeTime=bad", None, true)
            .await
            .0,
        400
    );
    assert_eq!(
        operator
            .send(
                "GET",
                "/api/v1/operator/history?beforeTime=bad&beforeKey=content:1",
                None,
                true
            )
            .await
            .0,
        400
    );
    let history = operator
        .send("GET", "/api/v1/operator/history", None, true)
        .await;
    assert_eq!(history.0, 200);
    let history_text = history.1.to_string();
    for action in [
        "approve", "reject", "import", "stage", "activate", "withdraw",
    ] {
        assert!(
            history.1["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["action"] == action),
            "missing {action}"
        );
    }
    assert!(!history_text.contains("serverOnly"));
    assert!(!history_text.contains("password"));
    assert!(!history_text.contains("csrfToken"));
    // Tie timestamps exercise the secondary key and boundaries across tables.
    db.execute_unprepared("INSERT INTO content_audit(action,actor,reason,generation,created_at) SELECT 'stage','pagination-test','pagination-'||n,2,'2026-10-08T00:00:00Z'::timestamptz FROM generate_series(1,25) n").await.unwrap();
    let first = operator
        .send("GET", "/api/v1/operator/history", None, true)
        .await;
    assert_eq!(first.0, 200);
    assert_eq!(first.1["items"].as_array().unwrap().len(), 20);
    let mut seen = std::collections::HashSet::new();
    let mut page = first.1;
    loop {
        for item in page["items"].as_array().unwrap() {
            assert!(
                seen.insert(item["key"].as_str().unwrap().to_owned()),
                "duplicate page entry"
            );
        }
        if page["next"].is_null() {
            break;
        }
        let mut url = url::Url::parse("http://test/api/v1/operator/history").unwrap();
        url.query_pairs_mut()
            .append_pair("beforeTime", page["next"]["beforeTime"].as_str().unwrap())
            .append_pair("beforeKey", page["next"]["beforeKey"].as_str().unwrap());
        let result = operator
            .send(
                "GET",
                &format!("{}?{}", url.path(), url.query().unwrap()),
                None,
                true,
            )
            .await;
        assert_eq!(result.0, 200);
        page = result.1;
    }
    assert_eq!(
        seen.len(),
        25 + history.1["items"].as_array().unwrap().len()
    );
    assert_eq!(
        latest.1["lessons"]
            .as_array()
            .unwrap()
            .iter()
            .find(|lesson| lesson["revision"] == 1)
            .unwrap()["withdrawn"],
        true
    );
    let account_endpoint = "/api/v1/operator/accounts/token";
    let invitation = json!({"email":"new@example.test","kind":"invite","operator":false,"reason":"new learner test"});
    assert_eq!(
        learner
            .send("POST", account_endpoint, Some(invitation.clone()), true)
            .await
            .0,
        403
    );
    assert_eq!(
        operator
            .send("POST", account_endpoint, Some(invitation.clone()), false)
            .await
            .0,
        403
    );
    assert_eq!(
        visitor
            .send("GET", "/api/v1/operator/accounts", None, true)
            .await
            .0,
        401
    );
    assert_eq!(
        learner
            .send("GET", "/api/v1/operator/accounts", None, true)
            .await
            .0,
        403
    );
    let first_invite = operator
        .send("POST", account_endpoint, Some(invitation.clone()), true)
        .await;
    assert_eq!(first_invite.0, 200);
    assert_eq!(first_invite.1["expiresInSeconds"], 172800);
    let second_invite = operator
        .send("POST", account_endpoint, Some(invitation), true)
        .await;
    assert_eq!(second_invite.0, 200);
    assert_ne!(first_invite.1["token"], second_invite.1["token"]);
    let mut invitee = Browser::new(app.clone()).await;
    let mut accept = json!({"token":first_invite.1["token"],"email":"new@example.test","displayName":"New account","password":"correct horse brioche fromage"});
    assert_eq!(
        invitee
            .send(
                "POST",
                "/api/v1/auth/accept-invite",
                Some(accept.clone()),
                true
            )
            .await
            .0,
        400
    );
    accept["token"] = second_invite.1["token"].clone();
    assert_eq!(
        invitee
            .send(
                "POST",
                "/api/v1/auth/accept-invite",
                Some(accept.clone()),
                true
            )
            .await
            .0,
        200
    );
    assert_eq!(
        invitee
            .send("POST", "/api/v1/auth/accept-invite", Some(accept), true)
            .await
            .0,
        400
    );
    let reset =
        json!({"email":"new@example.test","kind":"reset","operator":false,"reason":"reset test"});
    let reset_link = operator
        .send("POST", account_endpoint, Some(reset), true)
        .await;
    assert_eq!(reset_link.0, 200);
    assert_eq!(reset_link.1["expiresInSeconds"], 1800);
    let mut password_reset = Browser::new(app.clone()).await;
    assert_eq!(password_reset.send("POST","/api/v1/auth/reset-password",Some(json!({"token":reset_link.1["token"],"password":"changed horse brioche fromage"})),true).await.0,200);
    assert_eq!(invitee.send("GET", "/api/v1/me", None, true).await.0, 401);
    let mut invalid_invite =
        json!({"email":"another@example.test","kind":"invite","operator":false,"reason":""});
    assert_eq!(
        operator
            .send("POST", account_endpoint, Some(invalid_invite.clone()), true)
            .await
            .0,
        400
    );
    invalid_invite["reason"] = json!("check");
    invalid_invite["kind"] = json!("reset");
    invalid_invite["operator"] = json!(true);
    assert_eq!(
        operator
            .send("POST", account_endpoint, Some(invalid_invite), true)
            .await
            .0,
        400
    );
    let account_history = operator
        .send("GET", "/api/v1/operator/history", None, true)
        .await;
    let audit_count = db
        .query_one_raw(Statement::from_string(
            DbBackend::Postgres,
            "SELECT count(*) AS n FROM account_admin_audit".to_owned(),
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap();
    assert_eq!(audit_count, 3);
    let operator_invite=operator.send("POST",account_endpoint,Some(json!({"email":"invited-operator@example.test","kind":"invite","operator":true,"reason":"add another operator test"})),true).await;
    assert_eq!(operator_invite.0, 200);
    let mut new_operator = Browser::new(app.clone()).await;
    let accepted=new_operator.send("POST","/api/v1/auth/accept-invite",Some(json!({"token":operator_invite.1["token"],"email":"invited-operator@example.test","displayName":"Invited operator","password":"correct horse brioche fromage"})),true).await;
    assert_eq!(accepted.0, 200);
    assert_eq!(accepted.1["user"]["role"], "operator");
    assert_eq!(
        new_operator
            .send("GET", "/api/v1/operator/accounts", None, true)
            .await
            .0,
        200
    );
    let invite_role=db.query_one_raw(Statement::from_string(DbBackend::Postgres,"SELECT details->>'role' AS role FROM account_admin_audit WHERE target_email='invited-operator@example.test'".to_owned())).await.unwrap().unwrap().try_get::<String>("","role").unwrap();
    assert_eq!(invite_role, "operator");
    let history_text = account_history.1.to_string();
    assert!(!history_text.contains(first_invite.1["token"].as_str().unwrap()));
    assert!(!history_text.contains(second_invite.1["token"].as_str().unwrap()));
    assert!(!history_text.contains(reset_link.1["token"].as_str().unwrap()));
    // Search and keyset pagination must not expose private user fields.
    db.execute_unprepared("INSERT INTO users(email,password_hash,display_name,role,settings) SELECT 'page-'||n||'@example.test',password_hash,'Page account','learner',settings FROM users CROSS JOIN generate_series(1,25) n WHERE email='operator@example.test'").await.unwrap();
    let accounts = operator
        .send("GET", "/api/v1/operator/accounts?q=PAGE", None, true)
        .await;
    assert_eq!(accounts.0, 200);
    assert_eq!(accounts.1["items"].as_array().unwrap().len(), 20);
    assert!(!accounts.1.to_string().contains("password"));
    assert!(!accounts.1.to_string().contains("settings"));
    let next = operator
        .send(
            "GET",
            &format!(
                "/api/v1/operator/accounts?q=PAGE&afterId={}",
                accounts.1["nextId"].as_str().unwrap()
            ),
            None,
            true,
        )
        .await;
    assert_eq!(next.0, 200);
    assert_eq!(next.1["items"].as_array().unwrap().len(), 5);
    assert!(next.1["nextId"].is_null());
    assert!(accounts.1["items"].as_array().unwrap().iter().all(|first| {
        !next.1["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|second| first["id"] == second["id"])
    }));
    // Role management is authorized, optimistic, audited and protects the last operator.
    let users = operator
        .send("GET", "/api/v1/operator/accounts", None, true)
        .await;
    let lookup = |email: &str| {
        users.1["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|u| u["email"] == email)
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let first_id = lookup("operator@example.test");
    let second_id = lookup("invited-operator@example.test");
    let learner_id = lookup("learner@example.test");
    // Session controls never expose cookies/auth state and isolate the target account.
    let sessions_path = format!("/api/v1/operator/accounts/{learner_id}/sessions");
    assert_eq!(visitor.send("GET", &sessions_path, None, true).await.0, 401);
    assert_eq!(learner.send("GET", &sessions_path, None, true).await.0, 403);
    let initial = operator.send("GET", &sessions_path, None, true).await;
    assert_eq!(initial.0, 200);
    assert_eq!(initial.1["items"].as_array().unwrap().len(), 1);
    let initial_key = initial.1["items"][0]["id"].as_str().unwrap().to_owned();
    let mut learner_device = Browser::new(app.clone()).await;
    assert_eq!(learner_device.send("POST","/api/v1/auth/login",Some(json!({"email":"learner@example.test","password":"correct horse brioche fromage"})),true).await.0,200);
    let listed = operator.send("GET", &sessions_path, None, true).await;
    assert_eq!(listed.1["items"].as_array().unwrap().len(), 2);
    let key = listed.1["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] != initial_key)
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(key.len(), 64);
    assert!(
        !listed
            .1
            .to_string()
            .contains(learner_device.cookie.split('=').nth(1).unwrap())
    );
    for private in ["csrf", "password", "brioche.auth", "settings"] {
        assert!(!listed.1.to_string().contains(private));
    }
    let revoke = format!("{sessions_path}/{key}/revoke");
    let reason = json!({"reason":"revoke second learner device"});
    assert_eq!(
        learner
            .send("POST", &revoke, Some(reason.clone()), true)
            .await
            .0,
        403
    );
    assert_eq!(
        operator
            .send("POST", &revoke, Some(reason.clone()), false)
            .await
            .0,
        403
    );
    assert_eq!(
        operator
            .send(
                "POST",
                &format!("/api/v1/operator/accounts/{first_id}/sessions/{key}/revoke"),
                Some(reason.clone()),
                true
            )
            .await
            .0,
        404
    );
    assert_eq!(
        learner_device.send("GET", "/api/v1/me", None, true).await.0,
        200
    );
    assert_eq!(
        operator
            .send("POST", &revoke, Some(reason.clone()), true)
            .await
            .0,
        200
    );
    assert_eq!(
        operator.send("POST", &revoke, Some(reason), true).await.0,
        404
    );
    assert_eq!(
        learner_device.send("GET", "/api/v1/me", None, true).await.0,
        401
    );
    assert_eq!(learner.send("GET", "/api/v1/me", None, true).await.0, 200);
    assert_eq!(
        operator
            .send("GET", &format!("{sessions_path}?afterId=bad"), None, true)
            .await
            .0,
        400
    );
    // Synthetic opaque records test bounded paging and exclusion of expired sessions.
    db.execute_unprepared(&format!("INSERT INTO browser_sessions(id_hash,data,expires_at) SELECT lpad(to_hex(n),64,'0'),data,CURRENT_TIMESTAMP+interval '1 day' FROM browser_sessions CROSS JOIN generate_series(5000,5024) n WHERE id_hash='{initial_key}'; INSERT INTO browser_sessions(id_hash,data,expires_at) SELECT repeat('f',64),data,CURRENT_TIMESTAMP-interval '1 second' FROM browser_sessions WHERE id_hash='{initial_key}'")).await.unwrap();
    let first_page = operator.send("GET", &sessions_path, None, true).await;
    assert_eq!(first_page.1["items"].as_array().unwrap().len(), 20);
    let after = first_page.1["nextId"].as_str().unwrap();
    let second_page = operator
        .send(
            "GET",
            &format!("{sessions_path}?afterId={after}"),
            None,
            true,
        )
        .await;
    assert_eq!(second_page.1["items"].as_array().unwrap().len(), 6);
    assert!(second_page.1["nextId"].is_null());
    let all = first_page.1["items"]
        .as_array()
        .unwrap()
        .iter()
        .chain(second_page.1["items"].as_array().unwrap());
    let ids: std::collections::HashSet<_> = all.map(|s| s["id"].as_str().unwrap()).collect();
    assert_eq!(ids.len(), 26);
    assert!(!ids.contains("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"));
    let payload = json!({"expectedRole":"learner","role":"operator","reason":"promote learner"});
    let learner_path = format!("/api/v1/operator/accounts/{learner_id}/role");
    assert_eq!(
        learner
            .send("POST", &learner_path, Some(payload.clone()), true)
            .await
            .0,
        403
    );
    assert_eq!(
        visitor
            .send("POST", &learner_path, Some(payload.clone()), true)
            .await
            .0,
        401
    );
    assert_eq!(
        operator
            .send("POST", &learner_path, Some(payload.clone()), false)
            .await
            .0,
        403
    );
    assert_eq!(
        operator
            .send("POST", &learner_path, Some(payload.clone()), true)
            .await
            .0,
        200
    );
    // Existing learner session obtains current DB privileges; no re-login or token spoofing.
    assert_eq!(
        learner
            .send("GET", "/api/v1/operator/accounts", None, true)
            .await
            .0,
        200
    );
    assert_eq!(
        operator
            .send("POST", &learner_path, Some(payload), true)
            .await
            .0,
        409
    );
    let demote = json!({"expectedRole":"operator","role":"learner","reason":"demote learner"});
    assert_eq!(
        operator
            .send("POST", &learner_path, Some(demote.clone()), true)
            .await
            .0,
        200
    );
    assert_eq!(
        learner
            .send("GET", "/api/v1/operator/accounts", None, true)
            .await
            .0,
        403
    );
    // Two operators concurrently attempt self-demotion: exactly one survives.
    let first_path = format!("/api/v1/operator/accounts/{first_id}/role");
    let second_path = format!("/api/v1/operator/accounts/{second_id}/role");
    let (first, second) = tokio::join!(
        operator.send("POST", &first_path, Some(demote.clone()), true),
        new_operator.send("POST", &second_path, Some(demote.clone()), true)
    );
    assert!(matches!((first.0, second.0), (200, 409) | (409, 200)));
    let count = db
        .query_one_raw(Statement::from_string(
            DbBackend::Postgres,
            "SELECT count(*) AS n FROM users WHERE role='operator'".to_owned(),
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap();
    assert_eq!(count, 1);
    let changed = db.query_one_raw(Statement::from_string(DbBackend::Postgres,"SELECT count(*) AS n FROM account_admin_audit WHERE action='role' AND details ? 'from' AND details ? 'to'".to_owned())).await.unwrap().unwrap().try_get::<i64>("","n").unwrap();
    assert_eq!(changed, 3);
    let (survivor, removed, last_path) = if first.0 == 200 {
        (&mut new_operator, &mut operator, &second_path)
    } else {
        (&mut operator, &mut new_operator, &first_path)
    };
    assert_eq!(
        removed
            .send("POST", last_path, Some(demote.clone()), true)
            .await
            .0,
        403
    );
    assert_eq!(
        survivor.send("POST", last_path, Some(demote), true).await.0,
        409
    );
    // Earlier pagination fixtures deliberately sit in the future; skip those rows.
    let event = survivor
        .send(
            "GET",
            "/api/v1/operator/history?beforeTime=2026-10-08T00:00:00Z&beforeKey=account:0",
            None,
            true,
        )
        .await;
    assert!(
        event.1["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["action"] == "role")
    );
    let survivor_id = if first.0 == 200 {
        &second_id
    } else {
        &first_id
    };
    let own = survivor
        .send(
            "GET",
            &format!("/api/v1/operator/accounts/{survivor_id}/sessions"),
            None,
            true,
        )
        .await;
    let current = own.1["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["current"] == true)
        .unwrap()["id"]
        .as_str()
        .unwrap();
    let revoked = survivor
        .send(
            "POST",
            &format!("/api/v1/operator/accounts/{survivor_id}/sessions/{current}/revoke"),
            Some(json!({"reason":"revoke current operator browser"})),
            true,
        )
        .await;
    assert_eq!(revoked.0, 200);
    assert_eq!(revoked.1["current"], true);
    assert_eq!(survivor.send("GET", "/api/v1/me", None, true).await.0, 401);
    let count = db
        .query_one_raw(Statement::from_string(
            DbBackend::Postgres,
            "SELECT count(*) AS n FROM account_admin_audit WHERE action='sessions'".to_owned(),
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap();
    assert_eq!(count, 2);
    db.close().await.unwrap();
    admin
        .execute_unprepared(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    std::fs::remove_dir_all(&root).unwrap();
}
