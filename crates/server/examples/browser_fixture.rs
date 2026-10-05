//! Synthetic protocol data for a named loopback-only disposable database. Never production content.
use anyhow::{Result, ensure};
use axum::{body::Body, http::Request};
use brioche_server::{
    csrf::CsrfPolicy,
    development_source,
    identity::{self, Backend},
    project_source,
};
use http_body_util::BodyExt;
use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};
use sea_orm_migration::MigratorTrait;
use tower::ServiceExt;
#[path = "../tests/support/mod.rs"]
mod support;

#[tokio::main]
async fn main() -> Result<()> {
    let connection = std::env::var("TEST_DATABASE_URL")?;
    let url = url::Url::parse(&connection)?;
    ensure!(
        url.scheme() == "postgres"
            && url.host_str() == Some("127.0.0.1")
            && url.path() == "/brioche_browser_qa"
            && url.query().is_none(),
        "only named loopback disposable database is permitted"
    );
    let db = Database::connect(connection).await?;
    brioche_migration::Migrator::up(&db, None).await?;
    let source = development_source()?;
    let lesson = project_source(source.clone())?;
    db.execute_raw(Statement::from_sql_and_values(DbBackend::Postgres,"INSERT INTO lesson_revisions(lesson_id,revision,published,public_document,server_document) VALUES($1,$2,true,$3,$4)",[lesson.id.clone().into(),(lesson.revision as i32).into(),serde_json::to_value(lesson)?.into(),source.into()])).await?;
    support::fixture_release(&db).await;
    let backend = Backend::new(db.clone()).await?;
    let token = backend
        .issue_token("browser-qa@example.test", false, false)
        .await?;
    let app = identity::router(
        backend,
        CsrfPolicy::new(["http://127.0.0.1:5175".into()])?,
        false,
    );
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/csrf")
                .body(Body::empty())?,
        )
        .await?;
    let cookie = response.headers()["set-cookie"]
        .to_str()?
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let csrf: serde_json::Value =
        serde_json::from_slice(&response.into_body().collect().await?.to_bytes())?;
    let body = serde_json::json!({"email":"browser-qa@example.test","token":token,"password":"Browser protocol test only passphrase","displayName":"Browser QA"});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/accept-invite")
                .header("cookie", cookie)
                .header("origin", "http://127.0.0.1:5175")
                .header("x-csrf-token", csrf["csrfToken"].as_str().unwrap())
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body)?))?,
        )
        .await?;
    ensure!(
        response.status().is_success(),
        "synthetic account creation failed"
    );
    println!("Disposable browser fixture ready; unreviewed synthetic course, test account only.");
    Ok(())
}
