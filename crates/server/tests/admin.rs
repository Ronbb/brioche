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
    db.close().await.unwrap();
    admin
        .execute_unprepared(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    std::fs::remove_dir_all(&root).unwrap();
}
