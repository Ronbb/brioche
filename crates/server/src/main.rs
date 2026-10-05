use anyhow::{Context, Result, bail};
use brioche_server::{AppState, development_fixture, entity, project_source, router};
use sea_orm::{ActiveModelTrait, Database, Set};
use sea_orm_migration::MigratorTrait;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "brioche_server=info,tower_http=info".into()),
        )
        .init();
    let command = std::env::args().nth(1).unwrap_or_else(|| "serve".into());
    let fixture = std::env::var("CONTENT_MODE").unwrap_or_else(|_| "database".into()) == "fixture";
    let production =
        std::env::var("APP_ENV").unwrap_or_else(|_| "production".into()) != "development";
    if fixture && production {
        bail!("fixture content is only permitted with APP_ENV=development");
    }
    let db = if fixture && command == "serve" {
        None
    } else {
        Some(
            Database::connect(
                std::env::var("DATABASE_URL")
                    .context("DATABASE_URL is required for database mode")?,
            )
            .await
            .context("database connection failed")?,
        )
    };
    match command.as_str() {
        "migrate" => {
            brioche_migration::Migrator::up(db.as_ref().unwrap(), None).await?;
            tracing::info!("migrations complete");
            return Ok(());
        }
        "import" => {
            let file = std::env::args()
                .nth(2)
                .context("usage: brioche-server import <lesson.json> [--publish]")?;
            let source: serde_json::Value = serde_json::from_slice(&std::fs::read(file)?)?;
            let publish = std::env::args().any(|arg| arg == "--publish");
            if publish
                && source.pointer("/editorial/status").and_then(|v| v.as_str()) != Some("reviewed")
            {
                bail!("only reviewed content can be published");
            }
            let lesson = project_source(source.clone())?;
            brioche_server::grading::Grader::from_source(&lesson, &source)
                .map_err(|_| anyhow::anyhow!("invalid private grading rules"))?;
            entity::ActiveModel {
                lesson_id: Set(lesson.id.clone()),
                revision: Set(i32::try_from(lesson.revision)?),
                published: Set(publish),
                public_document: Set(serde_json::to_value(&lesson)?),
                server_document: Set(source),
            }
            .insert(db.as_ref().unwrap())
            .await?;
            tracing::info!("immutable lesson revision imported");
            return Ok(());
        }
        "serve" => {}
        _ => bail!("unknown command"),
    }
    let listener = tokio::net::TcpListener::bind(
        std::env::var("API_BIND").unwrap_or_else(|_| "0.0.0.0:3001".into()),
    )
    .await?;
    tracing::info!(address=%listener.local_addr()?,"API listening");
    axum::serve(
        listener,
        router(AppState {
            db,
            fixture: if fixture {
                Some(development_fixture()?)
            } else {
                None
            },
        }),
    )
    .with_graceful_shutdown(shutdown())
    .await?;
    Ok(())
}
async fn shutdown() {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("SIGTERM listener");
        tokio::select! {_=tokio::signal::ctrl_c()=>{},_=terminate.recv()=>{}}
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
