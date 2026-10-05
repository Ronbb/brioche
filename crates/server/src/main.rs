use anyhow::{Context, Result, bail};
use brioche_server::{AppState, development_fixture, entity, project_source, router};
use sea_orm::{ActiveModelTrait, ConnectOptions, ConnectionTrait, Database, Set};
use sea_orm_migration::MigratorTrait;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "brioche_server=info,tower_http=info".into()),
        )
        .init();
    let command = std::env::args().nth(1).unwrap_or_else(|| "serve".into());
    if command == "audio-check" {
        let args: Vec<String> = std::env::args().skip(2).collect();
        if args.len() != 2 {
            bail!("usage: brioche-server audio-check <recording-file> <audio/mpeg|audio/wav>");
        }
        let (bytes, info) = tokio::task::spawn_blocking(move || {
            brioche_server::audio::inspect_file(std::path::Path::new(&args[0]), &args[1])
                .with_context(|| format!("{}: invalid recording", args[0]))
        })
        .await??;
        use sha2::{Digest, Sha256};
        println!(
            "{}",
            serde_json::json!({
                "sha256": format!("{:x}", Sha256::digest(&bytes)),
                "byteLength": bytes.len(),
                "durationMs": info.duration_ms,
                "sampleRate": info.sample_rate,
                "channels": info.channels,
            })
        );
        return Ok(());
    }
    if matches!(command.as_str(), "check" | "check-release") {
        let args: Vec<String> = std::env::args().skip(2).collect();
        if args.len() != 1 {
            bail!("usage: brioche-server {command} <file.json>");
        }
        let path = &args[0];
        if command == "check-release" {
            let manifest = brioche_server::content::ReleaseManifest::deserialize_file(path)?;
            manifest.validate().map_err(|_| anyhow::anyhow!("{path}: invalid directory IDs, revisions, schema version or duplicate references"))?;
        } else {
            let source: serde_json::Value = brioche_server::author_json::load(path)?;
            brioche_server::media::source_asset_refs(&source)
                .with_context(|| format!("{path}: invalid asset references"))?;
            brioche_server::recording::source_audio_refs(&source)
                .with_context(|| format!("{path}: invalid recording references"))?;
            let lesson = project_source(source.clone())
                .with_context(|| format!("{path}: invalid lesson structure or references"))?;
            brioche_server::grading::Grader::from_source(&lesson, &source).map_err(|_| {
                anyhow::anyhow!(
                    "{path}: serverOnly.grading: invalid or inconsistent private grading rules"
                )
            })?;
        }
        println!(
            "Structural checks passed. Publication still requires registered media, editorial review and release-stage validation."
        );
        return Ok(());
    }
    let fixture = std::env::var("CONTENT_MODE").unwrap_or_else(|_| "database".into()) == "fixture";
    let production =
        std::env::var("APP_ENV").unwrap_or_else(|_| "production".into()) != "development";
    if fixture && production {
        bail!("fixture content is only permitted with APP_ENV=development");
    }
    let db = if fixture && command == "serve" {
        None
    } else {
        let mut options = ConnectOptions::new(
            std::env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL is required for database mode"))?,
        );
        options
            .sqlx_logging(false)
            .max_connections(10)
            .connect_timeout(std::time::Duration::from_secs(5))
            .acquire_timeout(std::time::Duration::from_secs(5));
        Some(
            Database::connect(options)
                .await
                .map_err(|_| anyhow::anyhow!("database connection failed"))?,
        )
    };
    match command.as_str() {
        "audio-import" => {
            let args: Vec<String> = std::env::args().skip(2).collect();
            if args.len() != 3 {
                bail!("usage: audio-import <bundle.json> <source-directory> <actor>");
            }
            let bundle = brioche_server::author_json::load(&args[0])?;
            brioche_server::recording::import_bundle(
                db.as_ref().unwrap(),
                bundle,
                std::path::Path::new(&args[1]),
                &brioche_server::media::media_root(),
                &args[2],
            )
            .await?;
            tracing::info!("immutable recording revisions imported");
            return Ok(());
        }
        "assets-import" => {
            let args: Vec<String> = std::env::args().skip(2).collect();
            if args.len() != 3 {
                bail!("usage: assets-import <bundle.json> <source-directory> <actor>");
            }
            let bundle = brioche_server::author_json::load(&args[0])?;
            brioche_server::media::import_bundle(
                db.as_ref().unwrap(),
                bundle,
                std::path::Path::new(&args[1]),
                &brioche_server::media::media_root(),
                &args[2],
            )
            .await?;
            tracing::info!("asset and character revisions imported");
            return Ok(());
        }
        "release-stage" => {
            let args: Vec<String> = std::env::args().skip(2).collect();
            if args.len() != 3 {
                bail!("usage: release-stage <manifest.json> <actor> <reason>");
            }
            let manifest = brioche_server::content::ReleaseManifest::deserialize_file(&args[0])?;
            brioche_server::content::stage(
                db.as_ref().unwrap(),
                &manifest,
                &args[1],
                &args[2],
                &brioche_server::media::media_root(),
            )
            .await?;
            tracing::info!("immutable directory release staged");
            return Ok(());
        }
        "release-activate" | "content-withdraw" => {
            let args: Vec<String> = std::env::args().skip(2).collect();
            let generation = if command == "release-activate" {
                if args.len() != 4 {
                    bail!(
                        "usage: release-activate <release-id> <expected-generation> <actor> <reason>"
                    );
                }
                brioche_server::content::activate(
                    db.as_ref().unwrap(),
                    &args[0],
                    args[1].parse()?,
                    &args[2],
                    &args[3],
                    &brioche_server::media::media_root(),
                )
                .await?
            } else {
                if args.len() != 5 {
                    bail!(
                        "usage: content-withdraw <lesson-id> <revision> <expected-generation> <actor> <reason>"
                    );
                }
                brioche_server::content::withdraw(
                    db.as_ref().unwrap(),
                    &args[0],
                    args[1].parse()?,
                    args[2].parse()?,
                    &args[3],
                    &args[4],
                )
                .await?
            };
            println!("content generation: {generation}");
            return Ok(());
        }
        "release-status" => {
            let row = db
                .as_ref()
                .unwrap()
                .query_one_raw(sea_orm::Statement::from_string(
                    sea_orm::DbBackend::Postgres,
                    "SELECT active_release,generation FROM content_state WHERE singleton",
                ))
                .await?
                .context("content state missing")?;
            let release: Option<String> = row.try_get("", "active_release")?;
            let generation: i64 = row.try_get("", "generation")?;
            println!(
                "{}",
                serde_json::json!({"activeRelease":release,"generation":generation})
            );
            return Ok(());
        }
        "migrate" => {
            brioche_migration::Migrator::up(db.as_ref().unwrap(), None).await?;
            tracing::info!("migrations complete");
            return Ok(());
        }
        "import" => {
            let file = std::env::args()
                .nth(2)
                .context("usage: brioche-server import <lesson.json>")?;
            let source: serde_json::Value = brioche_server::author_json::load(&file)?;
            let source =
                brioche_server::media::hydrate_source(db.as_ref().unwrap(), source).await?;
            let source =
                brioche_server::recording::hydrate_source(db.as_ref().unwrap(), source).await?;
            let publish = std::env::args().any(|arg| arg == "--publish");
            if publish {
                bail!("use release-stage and release-activate to publish an atomic directory");
            }
            let lesson = project_source(source.clone())?;
            brioche_server::grading::Grader::from_source(&lesson, &source)
                .map_err(|_| anyhow::anyhow!("invalid private grading rules"))?;
            entity::ActiveModel {
                lesson_id: Set(lesson.id.clone()),
                revision: Set(i32::try_from(lesson.revision)?),
                published: Set(false),
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
    let media_db = db.clone();
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
    if let Some(db) = media_db {
        app = app.merge(brioche_server::recording::router(
            db.clone(),
            brioche_server::media::media_root(),
        ));
        app = app.merge(brioche_server::media::router(
            db,
            brioche_server::media::media_root(),
        ));
    }
    axum::serve(listener, brioche_server::observability::observe(app))
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
