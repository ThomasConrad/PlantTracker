use axum::{
    extract::DefaultBodyLimit,
    http::{header, Method, StatusCode},
    middleware::from_fn,
    response::{Html, Json},
    routing::get,
    Router,
};
use clap::Parser;
use serde_json::{json, Value};
use std::{env, path::Path};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod app_state;
mod auth;
mod database;
mod handlers;
mod middleware;
mod models;
mod utils;

use app_state::AppState;
use handlers::{auth as auth_handlers, calendar, google_tasks, plants};
use planty_api::ApiDoc;
use utils::{
    google_tasks::GoogleTasksConfig, 
    token_refresh_scheduler::start_token_refresh_scheduler,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to run the server on
    #[arg(short, long, env = "PORT", default_value = "3000")]
    port: u16,

    /// Database URL (use "sqlite::memory:" for in-memory database)
    #[arg(
        short,
        long,
        env = "DATABASE_URL",
        default_value = "sqlite:planty.db"
    )]
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
    // Load environment variables FIRST
    dotenvy::dotenv().ok();
    
    let args = Args::parse();

    // Initialize tracing with specified log level (now reads RUST_LOG from .env)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}={},tower_http=debug",
                    env!("CARGO_PKG_NAME").replace('-', "_"),
                    args.log_level
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Database setup with custom URL
    let pool = database::create_pool_with_url(&args.database_url).await?;

    // Run migrations for production (embedded migrations)
    database::run_migrations(&pool).await?;

    // Create application state
    let mut app_state = AppState::new(pool.clone());

    // Start token refresh scheduler if Google Tasks is configured
    if let Ok(google_config) = GoogleTasksConfig::from_env() {
        tracing::info!("Starting Google OAuth token refresh scheduler");
        let notifier = start_token_refresh_scheduler(pool.clone(), google_config);
        app_state = app_state.with_token_notifier(notifier);
    } else {
        tracing::info!("Google Tasks not configured, skipping token refresh scheduler");
    }

    // Authentication setup
    let (session_layer, auth_layer) = auth::create_auth_layers(pool.clone());

    // CORS configuration - allow all origins in development
    let cors = if cfg!(debug_assertions) {
        // Development: Allow any origin
        CorsLayer::permissive()
    } else {
        // Production: Restrict to specific origins
        let allowed_origins = env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| {
                let host_ip = env::var("HOST_IP").unwrap_or_else(|_| "localhost".to_string());
                format!("http://{}:3000,http://127.0.0.1:3000", host_ip)
            })
            .split(',')
            .filter_map(|origin| origin.trim().parse().ok())
            .collect::<Vec<_>>();
        
        CorsLayer::new()
            .allow_origin(allowed_origins)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::COOKIE,
                header::SET_COOKIE,
            ])
            .allow_credentials(true)
    };

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
        .nest("/calendar", calendar::routes())
        .nest("/google-tasks", google_tasks::routes())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/openapi.json", get(|| async { Json(ApiDoc::openapi()) }))
        .with_state(app_state);

    // Build main application router
    let app = if serve_frontend {
        let frontend_dir_clone = args.frontend_dir.clone();
        Router::new()
            .nest("/api/v1", api_router)
            .route("/api/health", get(health_check))
            // Handle unknown API routes with 404
            .route("/api/*path", get(api_not_found))
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

    // Configure file upload limit from environment
    let max_file_size = env::var("MAX_FILE_SIZE")
        .unwrap_or_else(|_| "10485760".to_string()) // 10MB default
        .parse::<usize>()
        .unwrap_or(10 * 1024 * 1024);
    
    tracing::info!("Max file upload size: {} bytes ({:.1} MB)", max_file_size, max_file_size as f64 / 1024.0 / 1024.0);

    let app = app.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(from_fn(crate::middleware::logging::log_errors))
            .layer(cors)
            .layer(DefaultBodyLimit::max(max_file_size))
            .layer(auth_layer)
            .layer(session_layer),
    );

    // Start server
    let addr = format!("0.0.0.0:{}", args.port);

    tracing::info!("Planty API starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "planty-api",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// API 404 handler - returns proper 404 for unknown API routes
async fn api_not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}

// SPA fallback handler - serves index.html for unmatched routes
async fn serve_spa_fallback(frontend_dir: String) -> Result<Html<String>, StatusCode> {
    let index_path = format!("{frontend_dir}/index.html");

    tokio::fs::read_to_string(&index_path).await.map_or_else(
        |_| {
            tracing::error!("Failed to read index.html from: {}", index_path);
            Err(StatusCode::NOT_FOUND)
        },
        |content| Ok(Html(content)),
    )
}
