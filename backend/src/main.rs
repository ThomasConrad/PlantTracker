use axum::{
    extract::DefaultBodyLimit,
    http::{header, Method, StatusCode, Uri},
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use clap::Parser;
use serde_json::{json, Value};
use std::{env, path::Path};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod database;
mod handlers;
mod middleware;
mod models;
mod utils;

use handlers::{auth as auth_handlers, photos, plants, tracking};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to run the server on
    #[arg(short, long, env = "PORT", default_value = "3000")]
    port: u16,

    /// Database URL (use "sqlite::memory:" for in-memory database)
    #[arg(short, long, env = "DATABASE_URL", default_value = "sqlite:plant_tracker.db")]
    database_url: String,

    /// Frontend directory path
    #[arg(short, long, env = "FRONTEND_DIR", default_value = "../frontend/dist")]
    frontend_dir: String,

    /// Log level
    #[arg(short, long, env = "RUST_LOG", default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    // Initialize tracing with specified log level
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}={},tower_http=debug", env!("CARGO_PKG_NAME").replace("-", "_"), args.log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Database setup with custom URL
    let pool = database::create_pool_with_url(&args.database_url).await?;

    // Authentication setup
    let (session_layer, auth_layer) = auth::create_auth_layers(pool.clone());

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    // Get the frontend dist directory path
    let serve_frontend = Path::new(&args.frontend_dir).exists();

    if serve_frontend {
        tracing::info!("Serving frontend from: {}", args.frontend_dir);
    } else {
        tracing::warn!(
            "Frontend directory not found: {}. API-only mode.",
            args.frontend_dir
        );
    }

    // Build API router
    let api_router = Router::new()
        .route("/health", get(health_check))
        .nest("/auth", auth_handlers::routes())
        .nest("/plants", plants::routes())
        .nest("/photos", photos::routes())
        .nest("/tracking", tracking::routes())
        .with_state(pool);

    // Build main application router
    let app = if serve_frontend {
        let frontend_dir_clone = args.frontend_dir.clone();
        Router::new()
            .nest("/api/v1", api_router)
            .route("/api/health", get(health_check))
            .fallback_service(
                ServeDir::new(&args.frontend_dir)
                    .append_index_html_on_directories(true)
                    .fallback(get(move || serve_spa_fallback(frontend_dir_clone))),
            )
    } else {
        Router::new()
            .route("/", get(health_check))
            .nest("/v1", api_router)
    };

    let app = app.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(cors)
            .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB max
            .layer(auth_layer)
            .layer(session_layer),
    );

    // Start server
    let addr = format!("0.0.0.0:{}", args.port);

    tracing::info!("Plant Tracker API starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "plant-tracker-api",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// SPA fallback handler - serves index.html for unmatched routes
async fn serve_spa_fallback(frontend_dir: String) -> Result<Html<String>, StatusCode> {
    let index_path = format!("{}/index.html", frontend_dir);

    match tokio::fs::read_to_string(&index_path).await {
        Ok(content) => Ok(Html(content)),
        Err(_) => {
            tracing::error!("Failed to read index.html from: {}", index_path);
            Err(StatusCode::NOT_FOUND)
        }
    }
}
