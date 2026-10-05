//! Runs only against an explicitly supplied test database, in a disposable schema.
use brioche_server::{AppState, development_fixture, entity, router};
use sea_orm::{ActiveModelTrait, ConnectOptions, ConnectionTrait, Database, Set};
use sea_orm_migration::MigratorTrait;
use tower::ServiceExt;

#[tokio::test]
#[ignore = "set TEST_DATABASE_URL to a dedicated PostgreSQL database"]
async fn migrations_publication_and_revision_uniqueness() {
    let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL is required");
    let admin = Database::connect(&url).await.unwrap();
    let schema = format!(
        "brioche_test_{}",
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
    brioche_migration::Migrator::up(&db, None).await.unwrap();
    let lesson = development_fixture().unwrap();
    let row = entity::ActiveModel {
        lesson_id: Set(lesson.id.clone()),
        revision: Set(1),
        published: Set(false),
        public_document: Set(serde_json::to_value(&lesson).unwrap()),
        server_document: Set(serde_json::json!({"serverOnly":{"grading":"private"}})),
    };
    row.clone().insert(&db).await.unwrap();
    assert!(
        row.insert(&db).await.is_err(),
        "duplicate revision must not overwrite"
    );
    let request = || {
        axum::http::Request::builder()
            .uri("/api/lessons/a1-bakery-buy-breakfast")
            .body(axum::body::Body::empty())
            .unwrap()
    };
    let app = router(AppState {
        db: Some(db.clone()),
        fixture: None,
    });
    assert_eq!(
        app.clone().oneshot(request()).await.unwrap().status(),
        404,
        "draft must not be public"
    );
    let mut lesson = lesson;
    lesson.revision = 2;
    entity::ActiveModel {
        lesson_id: Set(lesson.id.clone()),
        revision: Set(2),
        published: Set(true),
        public_document: Set(serde_json::to_value(&lesson).unwrap()),
        server_document: Set(serde_json::json!({"serverOnly":{"grading":"private"}})),
    }
    .insert(&db)
    .await
    .unwrap();
    let response = app.oneshot(request()).await.unwrap();
    assert_eq!(response.status(), 200);
    use http_body_util::BodyExt;
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let public: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(public["revision"], 2);
    assert!(public.get("serverOnly").is_none());
    brioche_migration::Migrator::down(&db, None).await.unwrap();
    drop(db);
    admin
        .execute_unprepared(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
}
