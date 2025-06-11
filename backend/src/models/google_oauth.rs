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
    #[schema(example = "Google Calendar integration configured successfully")]
    pub message: String,
    pub connected_at: DateTime<Utc>,
    pub scopes: Vec<String>,
}

/// Google Calendar connection status
#[derive(Debug, Serialize, ToSchema)]
pub struct GoogleCalendarStatus {
    pub connected: bool,
    pub connected_at: Option<DateTime<Utc>>,
    pub scopes: Option<Vec<String>>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Google Calendar event creation request
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateGoogleCalendarEventRequest {
    #[schema(example = "💧 Water Fiddle Leaf Fig")]
    pub summary: String,
    #[schema(example = "Time to water your Fiddle Leaf Fig. Remember to check soil moisture first.")]
    pub description: Option<String>,
    #[schema(example = "2024-01-15T10:00:00Z")]
    pub start_time: DateTime<Utc>,
    #[schema(example = "2024-01-15T11:00:00Z")]
    pub end_time: DateTime<Utc>,
    #[schema(example = "Plant Care")]
    pub calendar_id: Option<String>, // If None, uses primary calendar
    #[schema(example = "Plant care reminder")]
    pub location: Option<String>,
}

/// Google Calendar sync request
#[derive(Debug, Deserialize, ToSchema)]
pub struct SyncPlantRemindersRequest {
    /// Number of days in the future to sync events
    #[schema(example = 365, minimum = 1, maximum = 730)]
    pub days_ahead: Option<i32>,
    /// Whether to replace existing events or only add new ones
    #[schema(example = false)]
    pub replace_existing: Option<bool>,
}