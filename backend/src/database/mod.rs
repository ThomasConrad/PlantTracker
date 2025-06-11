use anyhow::Result;
use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use std::env;

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
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:plant_tracker.db".to_string());
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
pub async fn create_pool_with_url(database_url: &str) -> Result<DatabasePool> {
    tracing::info!("Connecting to database: {}", database_url);

    let pool = SqlitePool::connect(database_url).await?;

    tracing::info!("Database connected and ready");
    Ok(pool)
}

/// Runs database migrations on the given pool.
///
/// # Arguments
///
/// * `pool` - The database connection pool
///
/// # Errors
///
/// This function will return an error if:
/// - Database migrations fail to run
pub async fn run_migrations(pool: &DatabasePool) -> Result<()> {
    tracing::info!("Running database migrations");
    sqlx::migrate!("./migrations").run(pool).await?;
    tracing::info!("Database migrations applied");
    Ok(())
}

pub mod google_oauth;
pub mod photos;
pub mod plants;
pub mod tracking;
pub mod users;
