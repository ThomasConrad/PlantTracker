use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::{sleep_until, Duration, Instant};

use crate::database::{google_oauth, DatabasePool};
use crate::utils::google_tasks::{refresh_access_token, GoogleTasksConfig};
use crate::utils::errors::Result;

/// Background task scheduler for refreshing Google OAuth tokens
pub struct TokenRefreshScheduler {
    pool: DatabasePool,
    config: GoogleTasksConfig,
    notify: Arc<Notify>,
}

impl TokenRefreshScheduler {
    pub fn new(pool: DatabasePool, config: GoogleTasksConfig) -> Self {
        Self {
            pool,
            config,
            notify: Arc::new(Notify::new()),
        }
    }

    /// Get a handle to wake up the scheduler when new tokens are added
    pub fn get_notifier(&self) -> Arc<Notify> {
        Arc::clone(&self.notify)
    }

    /// Start the background token refresh task
    pub async fn start(self) {
        tracing::info!("Starting token refresh scheduler");
        
        loop {
            // First, refresh any tokens that need immediate refreshing
            if let Err(e) = self.refresh_expired_tokens().await {
                tracing::error!("Failed to refresh expired tokens: {}", e);
            }

            // Calculate when to wake up next
            let wake_time = match self.calculate_next_wake_time().await {
                Ok(Some(wake_time)) => wake_time,
                Ok(None) => {
                    // No tokens to refresh, wait indefinitely for notification
                    tracing::info!("No tokens to refresh, waiting for notification");
                    self.notify.notified().await;
                    continue;
                }
                Err(e) => {
                    tracing::error!("Failed to calculate next wake time: {}", e);
                    // Fallback: check again in 1 hour
                    Instant::now() + Duration::from_secs(3600)
                }
            };

            // Sleep until the next wake time or until notified
            tracing::info!("Token scheduler sleeping until: {:?}", wake_time);
            
            tokio::select! {
                _ = sleep_until(wake_time) => {
                    tracing::info!("Token scheduler woke up due to timer");
                }
                _ = self.notify.notified() => {
                    tracing::info!("Token scheduler woke up due to notification");
                }
            }
        }
    }

    /// Refresh all tokens that are expiring soon
    async fn refresh_expired_tokens(&self) -> Result<()> {
        let tokens = google_oauth::get_tokens_needing_refresh(&self.pool).await?;
        
        if tokens.is_empty() {
            tracing::debug!("No tokens need refreshing");
            return Ok(());
        }

        tracing::info!("Found {} tokens that need refreshing", tokens.len());

        for token in tokens {
            if let Some(refresh_token) = &token.refresh_token {
                match refresh_access_token(&self.config, refresh_token).await {
                    Ok((new_access_token, new_expires_at)) => {
                        // Update the token in the database
                        match google_oauth::update_access_token(
                            &self.pool,
                            &token.user_id,
                            &new_access_token,
                            new_expires_at,
                        ).await {
                            Ok(_) => {
                                tracing::info!("Successfully refreshed token for user: {}", token.user_id);
                            }
                            Err(e) => {
                                tracing::error!("Failed to update refreshed token for user {}: {}", token.user_id, e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to refresh token for user {}: {}", token.user_id, e);
                        // Optionally, we could delete invalid refresh tokens here
                    }
                }
            } else {
                tracing::warn!("Token for user {} has no refresh token", token.user_id);
            }
        }

        Ok(())
    }

    /// Calculate when the scheduler should wake up next
    async fn calculate_next_wake_time(&self) -> Result<Option<Instant>> {
        let next_expiration = google_oauth::get_next_token_expiration(&self.pool).await?;
        
        if let Some(expiration) = next_expiration {
            // Wake up 10 minutes before the token expires
            let wake_time = expiration - chrono::Duration::minutes(10);
            let now = Utc::now();
            
            if wake_time <= now {
                // Token needs refreshing now
                return Ok(Some(Instant::now()));
            }
            
            // Convert to tokio Instant
            let duration_until_wake = wake_time - now;
            let duration_std = std::time::Duration::from_secs(
                duration_until_wake.num_seconds().max(0) as u64
            );
            
            return Ok(Some(Instant::now() + duration_std));
        }
        
        Ok(None)
    }
}

/// Start the token refresh scheduler as a background task
pub fn start_token_refresh_scheduler(
    pool: DatabasePool,
    config: GoogleTasksConfig,
) -> Arc<Notify> {
    let scheduler = TokenRefreshScheduler::new(pool, config);
    let notifier = scheduler.get_notifier();
    
    tokio::spawn(async move {
        scheduler.start().await;
    });
    
    notifier
}