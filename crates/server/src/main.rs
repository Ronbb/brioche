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
        "invite" | "reset-password" => {
            let email = std::env::args().nth(2).context("usage: brioche-server invite|reset-password <email> <private-output-file> [--operator]")?;
            let output = std::env::args()
                .nth(3)
                .context("private output file is required")?;
            let base = std::env::var("PUBLIC_APP_URL").context("PUBLIC_APP_URL is required")?;
            brioche_server::csrf::CsrfPolicy::new([base.clone()])?;
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options
                .open(&output)
                .context("cannot create private output file")?;
            let backend =
                brioche_server::identity::Backend::new(db.as_ref().unwrap().clone()).await?;
            let reset = command == "reset-password";
            let token = backend
                .issue_token(&email, reset, std::env::args().any(|a| a == "--operator"))
                .await?;
            let mut link = url::Url::parse(&base)?;
            link.set_path(if reset { "/reset-password" } else { "/invite" });
            let fragment = url::form_urlencoded::Serializer::new(String::new())
                .append_pair("token", &token)
                .append_pair("email", &email)
                .finish();
            link.set_fragment(Some(&fragment));
            use std::io::Write;
            writeln!(file, "{link}")?;
            tracing::info!("one-time link written to the requested private file");
            return Ok(());
        }
        "serve" => {}
        _ => bail!("unknown command"),
    }
    let auth_router = if let Some(db) = &db {
        let public_url = std::env::var("PUBLIC_APP_URL")
            .context("PUBLIC_APP_URL is required in database mode")?;
        let mut origins = vec![public_url.clone()];
        if let Ok(additional) = std::env::var("ADDITIONAL_APP_ORIGINS") {
            origins.extend(
                additional
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned),
            );
        }
        let policy = brioche_server::csrf::CsrfPolicy::new(origins)?;
        let secure = url::Url::parse(&public_url)?.scheme() == "https";
        let backend = brioche_server::identity::Backend::new(db.clone()).await?;
        Some(brioche_server::identity::router(backend, policy, secure))
    } else {
        None
    };
    let cleanup = db.clone().map(|db| tokio::spawn(async move {
        let store = brioche_server::session_store::PgSessionStore::new(db.clone());
        let mut timer = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            timer.tick().await;
            if store.delete_expired().await.is_err() { tracing::warn!("session cleanup unavailable"); }
            use sea_orm::ConnectionTrait;
            if db.execute_unprepared("DELETE FROM auth_throttle WHERE resets_at <= CURRENT_TIMESTAMP; DELETE FROM identity_tokens WHERE expires_at < CURRENT_TIMESTAMP - interval '7 days';").await.is_err() {
                tracing::warn!("identity cleanup unavailable");
            }
        }
    }));
    let listener = tokio::net::TcpListener::bind(
        std::env::var("API_BIND").unwrap_or_else(|_| "0.0.0.0:3001".into()),
    )
    .await?;
    tracing::info!(address=%listener.local_addr()?,"API listening");
    let mut app = router(AppState {
        db,
        fixture: if fixture {
            Some(development_fixture()?)
        } else {
            None
        },
    });
    if let Some(auth) = auth_router {
        app = app.merge(auth);
    }
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await?;
    if let Some(cleanup) = cleanup {
        cleanup.abort();
    }
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
