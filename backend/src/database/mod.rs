use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use std::env;
use anyhow::Result;

pub type DatabasePool = Pool<Sqlite>;

pub async fn create_pool() -> Result<DatabasePool> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:plant_tracker.db".to_string());
    
    tracing::info!("Connecting to database: {}", database_url);
    
    let pool = SqlitePool::connect(&database_url).await?;
    
    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    tracing::info!("Database connected and migrations applied");
    Ok(pool)
}

pub mod plants;
pub mod users;