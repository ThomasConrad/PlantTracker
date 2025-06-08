use axum::{
    extract::DefaultBodyLimit,
    http::{header, Method},
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::env;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod middleware;
mod models;
mod utils;

use handlers::{auth, plants, photos, tracking};

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

    // Build application router
    let app = Router::new()
        .route("/", get(health_check))
        .nest("/v1/auth", auth::routes())
        .nest("/v1/plants", plants::routes())
        .nest("/v1/photos", photos::routes())
        .nest("/v1/tracking", tracking::routes())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors)
                .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB max
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
