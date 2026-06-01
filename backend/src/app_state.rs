use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Notify;

use crate::database::DatabasePool;
use crate::llm::PlantCoach;

/// Application state that gets passed to all handlers
#[derive(Clone)]
pub struct AppState {
    pub pool: DatabasePool,
    pub token_refresh_notifier: Option<Arc<Notify>>,
    pub coach: Option<Arc<dyn PlantCoach>>,
    pub started_at: DateTime<Utc>,
}

impl AppState {
    pub fn new(pool: DatabasePool) -> Self {
        Self {
            pool,
            token_refresh_notifier: None,
            coach: None,
            started_at: Utc::now(),
        }
    }

    pub fn with_coach(mut self, coach: Arc<dyn PlantCoach>) -> Self {
        self.coach = Some(coach);
        self
    }

    pub fn with_token_notifier(mut self, notifier: Arc<Notify>) -> Self {
        self.token_refresh_notifier = Some(notifier);
        self
    }

    /// Notify the token refresh scheduler that new tokens have been added
    pub fn notify_token_added(&self) {
        if let Some(notifier) = &self.token_refresh_notifier {
            notifier.notify_one();
            tracing::debug!("Notified token refresh scheduler of new token");
        }
    }
}
