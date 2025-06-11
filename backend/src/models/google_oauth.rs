use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Google OAuth token stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleOAuthToken {
    pub user_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scope: String,
    pub token_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request payload for OAuth callback
#[derive(Debug, Deserialize, ToSchema)]
pub struct GoogleOAuthCallbackRequest {
    pub code: String,
    pub state: Option<String>,
}

/// Response containing OAuth authorization URL
#[derive(Debug, Serialize, ToSchema)]
pub struct GoogleOAuthUrlResponse {
    #[schema(example = "https://accounts.google.com/o/oauth2/auth?...")]
    pub auth_url: String,
    #[schema(example = "abc123xyz")]
    pub state: String,
}

/// Response after successful OAuth completion
#[derive(Debug, Serialize, ToSchema)]
pub struct GoogleOAuthSuccessResponse {
    pub success: bool,
    #[schema(example = "Google Tasks integration configured successfully")]
    pub message: String,
    pub connected_at: DateTime<Utc>,
    pub scopes: Vec<String>,
}

/// Google Tasks connection status
#[derive(Debug, Serialize, ToSchema)]
pub struct GoogleTasksStatus {
    pub connected: bool,
    pub connected_at: Option<DateTime<Utc>>,
    pub scopes: Option<Vec<String>>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Google Tasks task creation request
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateGoogleTaskRequest {
    #[schema(example = "💧 Water Fiddle Leaf Fig")]
    pub title: String,
    #[schema(example = "Time to water your Fiddle Leaf Fig. Remember to check soil moisture first.")]
    pub notes: Option<String>,
    #[schema(example = "2024-01-15T10:00:00Z")]
    pub due_time: DateTime<Utc>,
    #[schema(example = "Plant Care")]
    pub task_list_id: Option<String>, // If None, uses or creates "Plant Care" list
}

/// Google Tasks sync request
#[derive(Debug, Deserialize, ToSchema)]
pub struct SyncPlantTasksRequest {
    /// Number of days in the future to sync tasks
    #[schema(example = 365, minimum = 1, maximum = 730)]
    pub days_ahead: Option<i32>,
    /// Whether to replace existing tasks or only add new ones
    #[schema(example = false)]
    pub replace_existing: Option<bool>,
}