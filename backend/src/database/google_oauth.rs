use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::models::google_oauth::GoogleOAuthToken;
use crate::utils::errors::{AppError, Result};

/// Save or update Google OAuth token for a user
pub async fn save_oauth_token(
    pool: &SqlitePool,
    user_id: &str,
    access_token: &str,
    refresh_token: Option<&str>,
    expires_at: Option<DateTime<Utc>>,
    scope: &str,
) -> Result<GoogleOAuthToken> {
    let now = Utc::now();
    
    sqlx::query!(
        r#"
        INSERT INTO google_oauth_tokens (
            user_id, access_token, refresh_token, expires_at, scope, token_type, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'Bearer', ?, ?)
        ON CONFLICT(user_id) DO UPDATE SET
            access_token = excluded.access_token,
            refresh_token = excluded.refresh_token,
            expires_at = excluded.expires_at,
            scope = excluded.scope,
            updated_at = excluded.updated_at
        "#,
        user_id,
        access_token,
        refresh_token,
        expires_at,
        scope,
        now,
        now
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to save OAuth token for user {}: {}", user_id, e);
        AppError::Database(e)
    })?;

    // Fetch the inserted/updated token
    let token = get_oauth_token(pool, user_id).await?
        .ok_or_else(|| AppError::Internal {
            message: "Failed to retrieve saved token".to_string(),
        })?;

    tracing::info!("Saved OAuth token for user: {}", user_id);
    Ok(token)
}

/// Get Google OAuth token for a user
pub async fn get_oauth_token(pool: &SqlitePool, user_id: &str) -> Result<Option<GoogleOAuthToken>> {
    let row = sqlx::query!(
        r#"
        SELECT 
            user_id,
            access_token,
            refresh_token,
            expires_at,
            scope,
            token_type,
            created_at,
            updated_at
        FROM google_oauth_tokens 
        WHERE user_id = ?
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get OAuth token for user {}: {}", user_id, e);
        AppError::Database(e)
    })?;

    let token = if let Some(row) = row {
        Some(GoogleOAuthToken {
            user_id: row.user_id,
            access_token: row.access_token,
            refresh_token: row.refresh_token,
            expires_at: row.expires_at.map(|dt| DateTime::from_timestamp(dt.and_utc().timestamp(), 0).unwrap_or_else(Utc::now)),
            scope: row.scope,
            token_type: row.token_type,
            created_at: DateTime::from_timestamp(row.created_at.and_utc().timestamp(), 0).unwrap_or_else(Utc::now),
            updated_at: DateTime::from_timestamp(row.updated_at.and_utc().timestamp(), 0).unwrap_or_else(Utc::now),
        })
    } else {
        None
    };

    Ok(token)
}

/// Update access token for a user (used when refreshing)
pub async fn update_access_token(
    pool: &SqlitePool,
    user_id: &str,
    access_token: &str,
    expires_at: Option<DateTime<Utc>>,
) -> Result<()> {
    let now = Utc::now();
    
    sqlx::query!(
        r#"
        UPDATE google_oauth_tokens 
        SET access_token = ?, expires_at = ?, updated_at = ?
        WHERE user_id = ?
        "#,
        access_token,
        expires_at,
        now,
        user_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update access token for user {}: {}", user_id, e);
        AppError::Database(e)
    })?;

    tracing::info!("Updated access token for user: {}", user_id);
    Ok(())
}

/// Delete Google OAuth token for a user (disconnect)
pub async fn delete_oauth_token(pool: &SqlitePool, user_id: &str) -> Result<()> {
    let result = sqlx::query!(
        "DELETE FROM google_oauth_tokens WHERE user_id = ?",
        user_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete OAuth token for user {}: {}", user_id, e);
        AppError::Database(e)
    })?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound {
            resource: "Google Tasks connection".to_string(),
        });
    }

    tracing::info!("Deleted OAuth token for user: {}", user_id);
    Ok(())
}

/// Check if a user has a valid (non-expired) Google OAuth token
#[allow(dead_code)]
pub async fn has_valid_token(pool: &SqlitePool, user_id: &str) -> Result<bool> {
    let token = get_oauth_token(pool, user_id).await?;
    
    match token {
        Some(token) => {
            // Check if token is expired
            if let Some(expires_at) = token.expires_at {
                // Consider token expired if it expires within the next 5 minutes
                let expires_soon = expires_at < Utc::now() + chrono::Duration::minutes(5);
                Ok(!expires_soon)
            } else {
                // If no expiration time, assume it's valid
                Ok(true)
            }
        }
        None => Ok(false),
    }
}

/// Get all users who have Google Tasks integration enabled
#[allow(dead_code)]
pub async fn get_users_with_google_tasks(pool: &SqlitePool) -> Result<Vec<String>> {
    let user_ids = sqlx::query_scalar!(
        "SELECT user_id FROM google_oauth_tokens"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get users with Google Tasks: {}", e);
        AppError::Database(e)
    })?;

    Ok(user_ids)
}