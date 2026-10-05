//! Revision, ownership, retries and completion require real PostgreSQL transactions.
use axum::{Router, body::Body, http::Request};
use brioche_server::{
    csrf::CsrfPolicy,
    development_source,
    identity::{self, Backend},
    project_source,
};
use http_body_util::BodyExt;
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement,
};
use sea_orm_migration::MigratorTrait;
use serde_json::{Value, json};
use tower::ServiceExt;

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
        protected: bool,
    ) -> (u16, Value) {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header("cookie", &self.cookie);
        if protected {
            request = request
                .header("origin", "http://localhost:5173")
                .header("x-csrf-token", &self.csrf);
        }
        let request = if let Some(body) = body {
            request
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap()
        } else {
            request.body(Body::empty()).unwrap()
        };
        let response = self.app.clone().oneshot(request).await.unwrap();
        let status = response.status().as_u16();
        assert_eq!(
            response.headers().get("cache-control").unwrap(),
            "private, no-store"
        );
        if let Some(cookie) = response.headers().get("set-cookie") {
            self.cookie = cookie.to_str().unwrap().split(';').next().unwrap().into();
        }
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let result: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        if let Some(csrf) = result.get("csrfToken").and_then(Value::as_str) {
            self.csrf = csrf.into();
        }
        (status, result)
    }
    async fn account(&mut self, backend: &Backend, email: &str) {
        let token = backend.issue_token(email, false, false).await.unwrap();
        assert_eq!(self.send("POST","/api/v1/auth/accept-invite",Some(json!({"email":email,"token":token,"password":"learning integration only passphrase","displayName":"Learner"})),true).await.0,200);
    }
    async fn state(&mut self, id: &str) -> Value {
        let (status, result) = self
            .send(
                "GET",
                &format!("/api/v1/learning-sessions/{id}"),
                None,
                true,
            )
            .await;
        assert_eq!(status, 200);
        result["progress"].clone()
    }
    async fn step(&mut self, id: &str, step: &str, version: &Value, key: &str) -> (u16, Value) {
        self.send(
            "PUT",
            &format!("/api/v1/learning-sessions/{id}/steps/{step}"),
            Some(json!({"version":version,"idempotencyKey":key})),
            true,
        )
        .await
    }
    async fn attempt(
        &mut self,
        id: &str,
        exercise: &str,
        answer: Value,
        version: &Value,
        key: &str,
    ) -> (u16, Value) {
        self.send("POST",&format!("/api/v1/learning-sessions/{id}/attempts"),Some(json!({"version":version,"idempotencyKey":key,"exerciseId":exercise,"answer":answer})),true).await
    }
}
async fn publish(db: &DatabaseConnection, source: Value) {
    let lesson = project_source(source.clone()).unwrap();
    db.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"INSERT INTO lesson_revisions (lesson_id,revision,published,public_document,server_document) VALUES ($1,$2,true,$3,$4)",[lesson.id.clone().into(),(lesson.revision as i32).into(),serde_json::to_value(lesson).unwrap().into(),source.into()])).await.unwrap();
}
async fn count(db: &DatabaseConnection, table: &str) -> i64 {
    db.query_one_raw(Statement::from_string(
        DbBackend::Postgres,
        format!("SELECT count(*)::bigint AS n FROM {table}"),
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get("", "n")
    .unwrap()
}
fn start_body(lesson: &str, key: &str) -> Value {
    json!({"lessonId":lesson,"schemaVersion":"1.0","idempotencyKey":key})
}
#[tokio::test]
#[ignore = "set TEST_DATABASE_URL to a dedicated PostgreSQL database"]
async fn learning_revision_ownership_idempotency_and_completion() {
    let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL is required");
    let admin = Database::connect(&url).await.unwrap();
    let schema = format!(
        "learning_test_{}",
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
    options.set_schema_search_path(&schema).sqlx_logging(false);
    let db = Database::connect(options).await.unwrap();
    brioche_migration::Migrator::up(&db, None).await.unwrap();
    // Unreviewed fixture is published only into this isolated test schema, never a production import.
    let source = development_source().unwrap();
    let lesson_id = source["id"].as_str().unwrap();
    publish(&db, source.clone()).await;
    let backend = Backend::new(db.clone()).await.unwrap();
    let app = identity::router(
        backend.clone(),
        CsrfPolicy::new(["http://localhost:5173".into()]).unwrap(),
        false,
    );
    let mut a = Browser::new(app.clone()).await;
    assert_eq!(
        a.send("GET", "/api/v1/me/learning", None, true).await.0,
        401
    );
    assert_eq!(
        a.send(
            "POST",
            "/api/v1/learning-sessions",
            Some(start_body(lesson_id, "learning-start-001")),
            true
        )
        .await
        .0,
        401
    );
    a.account(&backend, "one@example.test").await;
    let mut b = Browser::new(app.clone()).await;
    b.account(&backend, "two@example.test").await;
    assert_eq!(
        a.send(
            "POST",
            "/api/v1/learning-sessions",
            Some(start_body(lesson_id, "learning-start-001")),
            false
        )
        .await
        .0,
        403
    );
    let mut invalid = start_body(lesson_id, "learning-start-001");
    invalid["schemaVersion"] = json!("99");
    assert_eq!(
        a.send("POST", "/api/v1/learning-sessions", Some(invalid), true)
            .await
            .0,
        400
    );
    assert_eq!(
        a.send(
            "POST",
            "/api/v1/learning-sessions",
            Some(start_body(lesson_id, "short")),
            true
        )
        .await
        .0,
        400
    );
    let initial = start_body(lesson_id, "learning-start-001");
    let (status, opened) = a
        .send(
            "POST",
            "/api/v1/learning-sessions",
            Some(initial.clone()),
            true,
        )
        .await;
    assert_eq!(status, 200);
    assert_eq!(opened["lesson"]["revision"], 1);
    let text = serde_json::to_string(&opened).unwrap();
    for forbidden in [
        "serverOnly",
        "correctOptionId",
        "correctTokenIds",
        "accepted",
        "editorial",
    ] {
        assert!(!text.contains(forbidden));
    }
    let id = opened["progress"]["id"].as_str().unwrap().to_owned();
    assert_eq!(
        a.send(
            "POST",
            "/api/v1/learning-sessions",
            Some(initial.clone()),
            true
        )
        .await
        .1,
        opened
    );
    assert_eq!(
        a.send(
            "POST",
            "/api/v1/learning-sessions",
            Some(start_body("different-lesson", "learning-start-001")),
            true
        )
        .await
        .0,
        409
    );
    let mut another_a = Browser::new(app.clone()).await;
    assert_eq!(another_a.send("POST","/api/v1/auth/login",Some(json!({"email":"one@example.test","password":"learning integration only passphrase"})),true).await.0,200);
    let (left, right) = tokio::join!(
        a.send(
            "POST",
            "/api/v1/learning-sessions",
            Some(start_body(lesson_id, "learning-start-002")),
            true
        ),
        another_a.send(
            "POST",
            "/api/v1/learning-sessions",
            Some(start_body(lesson_id, "learning-start-003")),
            true
        )
    );
    assert_eq!(left.0, 200);
    assert_eq!(right.0, 200);
    assert_eq!(left.1["progress"]["id"], right.1["progress"]["id"]);
    assert_eq!(count(&db, "learning_sessions").await, 1);
    assert_eq!(
        a.attempt(
            &id,
            "exercise-intention",
            json!({"kind":"choice","optionId":"request-bread"}),
            &json!(1),
            "attempt-too-early"
        )
        .await
        .0,
        409
    );
    assert_eq!(
        a.step(&id, "step-recap", &json!(1), "recap-too-early-1")
            .await
            .0,
        409
    );
    assert_eq!(
        a.send(
            "POST",
            &format!("/api/v1/learning-sessions/{id}/complete"),
            Some(json!({"version":1,"idempotencyKey":"complete-too-early"})),
            true
        )
        .await
        .0,
        409
    );
    let read_body = json!({"version":1,"idempotencyKey":"read-confirm-001"});
    let (status, read) = a
        .send(
            "PUT",
            &format!("/api/v1/learning-sessions/{id}/steps/step-read"),
            Some(read_body.clone()),
            true,
        )
        .await;
    assert_eq!(status, 200);
    assert_eq!(read["version"], 2);
    assert_eq!(read["lastStepId"], "step-explore");
    assert_eq!(
        a.send(
            "PUT",
            &format!("/api/v1/learning-sessions/{id}/steps/step-read"),
            Some(read_body),
            true
        )
        .await
        .1,
        read
    );
    assert_eq!(
        a.step(&id, "step-read", &json!(2), "read-confirm-001")
            .await
            .0,
        409
    );
    assert_eq!(
        a.step(&id, "step-read", &json!(2), "read-confirm-002")
            .await
            .1["version"],
        2
    );
    let alternative = source["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["id"] == "exercise-intention")
        .unwrap()["options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["id"] != "request-bread")
        .unwrap()["id"]
        .clone();
    let mut updated_source = source.clone();
    updated_source["revision"] = json!(2);
    updated_source["serverOnly"]["grading"]["exercise-intention"]["correctOptionId"] =
        alternative.clone();
    publish(&db, updated_source).await;
    assert_eq!(
        a.send(
            "POST",
            "/api/v1/learning-sessions",
            Some(start_body(lesson_id, "learning-start-004")),
            true
        )
        .await
        .1["lesson"]["revision"],
        1,
        "active session retains the old snapshot"
    );
    assert_eq!(
        a.attempt(
            &id,
            "unknown-exercise",
            json!({"kind":"choice","optionId":"request-bread"}),
            &json!(2),
            "unknown-exercise-1"
        )
        .await
        .0,
        404
    );
    assert_eq!(
        a.attempt(
            &id,
            "exercise-intention",
            json!({"kind":"text","text":"anything"}),
            &json!(2),
            "wrong-kind-test-1"
        )
        .await
        .0,
        400
    );
    assert_eq!(
        a.attempt(
            &id,
            "exercise-intention",
            json!({"kind":"choice","optionId":"request-bread","score":100}),
            &json!(2),
            "forged-score-001"
        )
        .await
        .0,
        422
    );
    assert_eq!(
        a.attempt(
            &id,
            "exercise-intention",
            json!({"kind":"choice","optionId":"unknown"}),
            &json!(2),
            "unknown-option-01"
        )
        .await
        .0,
        400
    );
    assert_eq!(count(&db, "exercise_attempts").await, 0);
    let hint_path = format!("/api/v1/learning-sessions/{id}/hints/exercise-article");
    let hint_body = json!({"version":2,"idempotencyKey":"hint-article-001"});
    let (status, hint) = a
        .send("POST", &hint_path, Some(hint_body.clone()), true)
        .await;
    assert_eq!(status, 200);
    assert_eq!(hint["progress"]["version"], 3);
    assert!(hint["hintZh"].as_str().unwrap().len() > 1);
    assert_eq!(
        a.send("POST", &hint_path, Some(hint_body), true).await.1,
        hint
    );
    assert_eq!(
        a.send(
            "POST",
            &format!("/api/v1/learning-sessions/{id}/hints/exercise-intention"),
            Some(json!({"version":3,"idempotencyKey":"hint-not-found-1"})),
            true
        )
        .await
        .0,
        404
    );
    let wrong_answer = json!({"kind":"choice","optionId":alternative});
    let (status, wrong) = a
        .attempt(
            &id,
            "exercise-intention",
            wrong_answer.clone(),
            &json!(3),
            "attempt-choice-01",
        )
        .await;
    assert_eq!(status, 200);
    assert_eq!(wrong["result"]["correct"], false);
    assert_eq!(
        a.attempt(
            &id,
            "exercise-intention",
            wrong_answer,
            &json!(3),
            "attempt-choice-01"
        )
        .await
        .1,
        wrong
    );
    assert_eq!(
        a.attempt(
            &id,
            "exercise-intention",
            json!({"kind":"choice","optionId":"request-bread"}),
            &json!(3),
            "attempt-choice-01"
        )
        .await
        .0,
        409
    );
    let (status, correct) = a
        .attempt(
            &id,
            "exercise-intention",
            json!({"kind":"choice","optionId":"request-bread"}),
            &json!(4),
            "attempt-choice-02",
        )
        .await;
    assert_eq!(status, 200);
    assert_eq!(
        correct["result"]["correct"], true,
        "grading uses revision 1, not the new rules"
    );
    let attempts = correct["progress"]["attempts"].as_array().unwrap();
    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[0]["attemptIndex"], 1);
    assert_eq!(attempts[0]["result"]["correct"], false);
    assert_eq!(attempts[1]["attemptIndex"], 2);
    let order = json!({"kind":"order","tokenIds":["please","bread","request"]}); // deliberately incorrect, but a valid attempt
    let concurrent_version = json!(5);
    let (left, right) = tokio::join!(
        a.attempt(
            &id,
            "exercise-article",
            json!({"kind":"text","text":"une"}),
            &concurrent_version,
            "attempt-article-1"
        ),
        another_a.attempt(
            &id,
            "exercise-order",
            order.clone(),
            &concurrent_version,
            "attempt-order-001"
        )
    );
    assert!((left.0 == 200 && right.0 == 409) || (left.0 == 409 && right.0 == 200));
    assert_eq!(count(&db, "exercise_attempts").await, 3);
    if left.0 == 409 {
        assert_eq!(
            a.attempt(
                &id,
                "exercise-article",
                json!({"kind":"text","text":"une"}),
                &json!(6),
                "attempt-article-2"
            )
            .await
            .0,
            200
        );
    } else {
        assert_eq!(
            a.attempt(
                &id,
                "exercise-order",
                order.clone(),
                &json!(6),
                "attempt-order-002"
            )
            .await
            .0,
            200
        );
    }
    let state = a.state(&id).await;
    assert_eq!(state["version"], 7);
    let hints = state["attempts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["exerciseId"] == "exercise-article")
        .unwrap();
    assert_eq!(hints["hintUsed"], true);
    assert_eq!(
        a.step(&id, "step-practice", &json!(7), "practice-confirm")
            .await
            .0,
        200
    );
    assert_eq!(
        a.step(&id, "step-recap", &json!(8), "recap-confirm-01")
            .await
            .0,
        200
    );
    let complete_path = format!("/api/v1/learning-sessions/{id}/complete");
    let complete_body = json!({"version":9,"idempotencyKey":"complete-final-01"});
    db.execute_unprepared("ALTER TABLE review_cards ADD CONSTRAINT test_card_failure CHECK (knowledge_id <> 'word-baguette') NOT VALID").await.unwrap();
    assert_eq!(
        a.send("POST", &complete_path, Some(complete_body.clone()), true)
            .await
            .0,
        503
    );
    assert_eq!(
        count(&db, "review_cards").await,
        0,
        "partial review insertion must roll back"
    );
    let failed = a.state(&id).await;
    assert_eq!(failed["version"], 9);
    assert!(failed["firstCompletedAt"].is_null());
    assert!(failed["completedAt"].is_null());
    db.execute_unprepared("ALTER TABLE review_cards DROP CONSTRAINT test_card_failure")
        .await
        .unwrap();
    let (status, completed) = a
        .send("POST", &complete_path, Some(complete_body.clone()), true)
        .await;
    assert_eq!(status, 200);
    assert_eq!(completed["version"], 10);
    assert!(completed["completedAt"].is_string());
    assert_eq!(count(&db, "review_cards").await, 3);
    let first = completed["firstCompletedAt"].clone();
    assert_eq!(
        a.send("POST", &complete_path, Some(complete_body), true)
            .await
            .1,
        completed
    );
    assert_eq!(
        a.send(
            "POST",
            &complete_path,
            Some(json!({"version":10,"idempotencyKey":"complete-repeat-1"})),
            true
        )
        .await
        .1["firstCompletedAt"],
        first
    );
    assert_eq!(
        a.attempt(
            &id,
            "exercise-intention",
            json!({"kind":"choice","optionId":"request-bread"}),
            &json!(10),
            "after-complete-1"
        )
        .await
        .0,
        409
    );
    for (method, path, body) in [
        ("GET", format!("/api/v1/learning-sessions/{id}"), None),
        (
            "PUT",
            format!("/api/v1/learning-sessions/{id}/steps/step-read"),
            Some(json!({"version":10,"idempotencyKey":"cross-owner-read"})),
        ),
        (
            "POST",
            complete_path.clone(),
            Some(json!({"version":10,"idempotencyKey":"cross-owner-done"})),
        ),
        (
            "POST",
            hint_path.clone(),
            Some(json!({"version":10,"idempotencyKey":"cross-owner-hint"})),
        ),
        (
            "POST",
            format!("/api/v1/learning-sessions/{id}/attempts"),
            Some(
                json!({"version":10,"idempotencyKey":"cross-owner-test","exerciseId":"exercise-intention","answer":{"kind":"choice","optionId":"request-bread"}}),
            ),
        ),
    ] {
        assert_eq!(b.send(method, &path, body, true).await.0, 404);
    }
    let (_, b_opened) = b
        .send("POST", "/api/v1/learning-sessions", Some(initial), true)
        .await;
    assert_ne!(b_opened["progress"]["id"], id);
    assert_eq!(b_opened["lesson"]["revision"], 2);
    assert_eq!(
        b.send("GET", "/api/v1/me/learning", None, true).await.1["completedLessons"],
        0
    );
    let (status, next) = a
        .send(
            "POST",
            "/api/v1/learning-sessions",
            Some(start_body(lesson_id, "new-run-start-01")),
            true,
        )
        .await;
    assert_eq!(status, 200);
    assert_eq!(next["lesson"]["revision"], 2);
    assert_eq!(next["progress"]["firstCompletedAt"], first);
    let new_id = next["progress"]["id"].as_str().unwrap().to_owned();
    assert_ne!(id, new_id);
    let mut version = json!(1);
    let (_, read) = a
        .step(&new_id, "step-read", &version, "second-read-0001")
        .await;
    version = read["version"].clone();
    for (exercise, answer, key) in [
        (
            "exercise-intention",
            json!({"kind":"choice","optionId":"request-bread"}),
            "second-choice-01",
        ),
        (
            "exercise-article",
            json!({"kind":"text","text":"une"}),
            "second-article-1",
        ),
        ("exercise-order", order, "second-order-001"),
    ] {
        let (status, response) = a.attempt(&new_id, exercise, answer, &version, key).await;
        assert_eq!(status, 200);
        if exercise == "exercise-intention" {
            assert_eq!(response["result"]["correct"], false);
        }
        version = response["progress"]["version"].clone();
    }
    for (step, key) in [
        ("step-practice", "second-practice-1"),
        ("step-recap", "second-recap-001"),
    ] {
        let (status, response) = a.step(&new_id, step, &version, key).await;
        assert_eq!(status, 200);
        version = response["version"].clone();
    }
    let (status, again) = a
        .send(
            "POST",
            &format!("/api/v1/learning-sessions/{new_id}/complete"),
            Some(json!({"version":version,"idempotencyKey":"second-complete1"})),
            true,
        )
        .await;
    assert_eq!(status, 200);
    assert_eq!(again["firstCompletedAt"], first);
    assert_eq!(
        count(&db, "review_cards").await,
        3,
        "shared knowledge does not reset or duplicate cards"
    );
    let latest:i32=db.query_one_raw(Statement::from_string(DbBackend::Postgres,"SELECT latest_completed_revision FROM lesson_progress WHERE first_completed_at IS NOT NULL")).await.unwrap().unwrap().try_get("","latest_completed_revision").unwrap();
    assert_eq!(latest, 2);
    assert_eq!(
        a.send("GET", "/api/v1/me/learning", None, true).await.1["completedLessons"],
        1
    );
    db.execute_unprepared("UPDATE lesson_revisions SET published=false")
        .await
        .unwrap();
    assert_eq!(
        a.send(
            "GET",
            &format!("/api/v1/learning-sessions/{id}"),
            None,
            true
        )
        .await
        .0,
        410
    );
    assert_eq!(
        a.send(
            "POST",
            "/api/v1/learning-sessions",
            Some(start_body(lesson_id, "learning-start-001")),
            true
        )
        .await
        .0,
        410
    );
    assert_eq!(
        a.attempt(
            &id,
            "exercise-intention",
            json!({"kind":"choice","optionId":alternative}),
            &json!(3),
            "attempt-choice-01"
        )
        .await
        .0,
        410,
        "withdrawal also blocks cached grading feedback"
    );
    assert_eq!(
        a.send("GET", "/api/v1/me/learning", None, true).await.1["items"],
        json!([])
    );
    // Cursor pagination has a fixed bound, stable tuple order and no foreign-user rows.
    for index in 0..25 {
        let mut page_source = source.clone();
        let page_id = format!("pagination-lesson-{index}");
        page_source["id"] = json!(page_id);
        publish(&db, page_source).await;
        assert_eq!(
            a.send(
                "POST",
                "/api/v1/learning-sessions",
                Some(start_body(&page_id, &format!("pagination-key-{index:03}"))),
                true
            )
            .await
            .0,
            200
        );
    }
    let (_, page1) = a.send("GET", "/api/v1/me/learning", None, true).await;
    assert_eq!(page1["items"].as_array().unwrap().len(), 20);
    let cursor = page1["nextCursor"].as_str().unwrap();
    let (_, page2) = a
        .send(
            "GET",
            &format!("/api/v1/me/learning?cursor={cursor}"),
            None,
            true,
        )
        .await;
    assert_eq!(page2["items"].as_array().unwrap().len(), 5);
    assert!(page2["nextCursor"].is_null());
    let mut ids = std::collections::BTreeSet::new();
    for page in [&page1, &page2] {
        for item in page["items"].as_array().unwrap() {
            assert!(ids.insert(item["sessionId"].as_str().unwrap()));
        }
    }
    assert_eq!(
        a.send("GET", "/api/v1/me/learning?cursor=broken", None, true)
            .await
            .0,
        400
    );
    brioche_migration::Migrator::down(&db, None).await.unwrap();
    drop(db);
    admin
        .execute_unprepared(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
}
