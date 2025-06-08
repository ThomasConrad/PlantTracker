use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use std::env;
use anyhow::Result;

pub type DatabasePool = Pool<Sqlite>;

/// Creates a database connection pool using the default `DATABASE_URL` environment variable.
///
/// # Errors
/// 
/// This function will return an error if:
/// - The database URL is invalid
/// - Connection to the database fails
/// - Database migrations fail to run
#[allow(dead_code)]
pub async fn create_pool() -> Result<DatabasePool> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:plant_tracker.db".to_string());
    create_pool_with_url(&database_url).await
}

/// Creates a database connection pool with a specific database URL.
///
/// # Arguments
/// 
/// * `database_url` - The database connection string
///
/// # Errors
/// 
/// This function will return an error if:
/// - The database URL is invalid
/// - Connection to the database fails  
/// - Database migrations fail to run
/// - Schema creation fails for in-memory databases
pub async fn create_pool_with_url(database_url: &str) -> Result<DatabasePool> {
    tracing::info!("Connecting to database: {}", database_url);
    
    let pool = SqlitePool::connect(database_url).await?;
    
    // Run migrations (skip for in-memory databases as they're ephemeral)
    if database_url.contains("memory:") {
        // For in-memory databases, create the schema manually
        tracing::info!("Creating in-memory database schema");
        create_schema(&pool).await?;
    } else {
        sqlx::migrate!("./migrations").run(&pool).await?;
        tracing::info!("Database migrations applied");
    }
    
    tracing::info!("Database connected and ready");
    Ok(pool)
}

async fn create_schema(pool: &DatabasePool) -> Result<()> {
    // Create tables for in-memory database
    sqlx::query(r"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            salt TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
    ").execute(pool).await?;

    sqlx::query(r"
        CREATE TABLE IF NOT EXISTS plants (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            name TEXT NOT NULL,
            genus TEXT NOT NULL,
            watering_interval_days INTEGER NOT NULL,
            fertilizing_interval_days INTEGER NOT NULL,
            last_watered TEXT,
            last_fertilized TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
        )
    ").execute(pool).await?;

    sqlx::query(r"
        CREATE TABLE IF NOT EXISTS custom_metrics (
            id TEXT PRIMARY KEY,
            plant_id TEXT NOT NULL,
            name TEXT NOT NULL,
            unit TEXT NOT NULL,
            data_type TEXT NOT NULL,
            FOREIGN KEY (plant_id) REFERENCES plants (id) ON DELETE CASCADE
        )
    ").execute(pool).await?;

    sqlx::query(r"
        CREATE TABLE IF NOT EXISTS tracking_entries (
            id TEXT PRIMARY KEY,
            plant_id TEXT NOT NULL,
            entry_type TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            value TEXT,
            notes TEXT,
            metric_id TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (plant_id) REFERENCES plants (id) ON DELETE CASCADE,
            FOREIGN KEY (metric_id) REFERENCES custom_metrics (id) ON DELETE SET NULL
        )
    ").execute(pool).await?;

    sqlx::query(r"
        CREATE TABLE IF NOT EXISTS photos (
            id TEXT PRIMARY KEY,
            plant_id TEXT NOT NULL,
            filename TEXT NOT NULL,
            original_filename TEXT NOT NULL,
            size INTEGER NOT NULL,
            content_type TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (plant_id) REFERENCES plants (id) ON DELETE CASCADE
        )
    ").execute(pool).await?;

    Ok(())
}

pub mod plants;
pub mod users;