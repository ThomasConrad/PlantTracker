use axum::{
    extract::DefaultBodyLimit,
    http::{header, Method, StatusCode, Uri},
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::{env, path::Path};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod middleware;
mod models;
mod utils;

use handlers::{auth, photos, plants, tracking};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "plant_tracker_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Database setup would go here
    // let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // let pool = sqlx::PgPool::connect(&database_url).await?;

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    // Get the frontend dist directory path
    let frontend_dir = env::var("FRONTEND_DIR").unwrap_or_else(|_| "../frontend/dist".to_string());
    let serve_frontend = Path::new(&frontend_dir).exists();

    if serve_frontend {
        tracing::info!("Serving frontend from: {}", frontend_dir);
    } else {
        tracing::warn!(
            "Frontend directory not found: {}. API-only mode.",
            frontend_dir
        );
    }

    // Build API router
    let api_router = Router::new()
        .route("/health", get(health_check))
        .nest("/auth", auth::routes())
        .nest("/plants", plants::routes())
        .nest("/photos", photos::routes())
        .nest("/tracking", tracking::routes());

    // Build main application router
    let app = if serve_frontend {
        Router::new()
            .nest("/api/v1", api_router)
            .route("/api/health", get(health_check))
            .fallback_service(
                ServeDir::new(&frontend_dir)
                    .append_index_html_on_directories(true)
                    .fallback(get(serve_spa_fallback)),
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
            .layer(DefaultBodyLimit::max(10 * 1024 * 1024)), // 10MB max
    );

    // Start server
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

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
async fn serve_spa_fallback(_uri: Uri) -> Result<Html<String>, StatusCode> {
    let frontend_dir = env::var("FRONTEND_DIR").unwrap_or_else(|_| "../frontend/dist".to_string());
    let index_path = format!("{}/index.html", frontend_dir);

    match tokio::fs::read_to_string(&index_path).await {
        Ok(content) => Ok(Html(content)),
        Err(_) => {
            tracing::error!("Failed to read index.html from: {}", index_path);
            Err(StatusCode::NOT_FOUND)
        }
    }
}
