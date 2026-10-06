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
        brioche_migration::Migrator::down(&db, Some(1))
            .await
            .is_err(),
        "operator recording audit cannot be removed by rollback"
    );
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
