use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use validator::Validate;

// Database row types

#[derive(Debug, Clone, FromRow)]
pub struct CoachConversationRow {
    pub id: String,
    pub plant_id: String,
    pub user_id: String,
    pub title: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct CoachMessageRow {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub image_url: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct CoachSuggestionRow {
    pub id: String,
    pub message_id: String,
    pub plant_id: String,
    pub suggestion_type: String,
    pub description: String,
    pub payload: String,
    pub status: String,
    pub applied_at: Option<String>,
    pub created_at: String,
}

// API response types

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CoachConversation {
    pub id: String,
    pub plant_id: String,
    pub title: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CoachMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub image_url: Option<String>,
    pub suggestions: Vec<CoachSuggestion>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CoachSuggestion {
    pub id: String,
    pub suggestion_type: String,
    pub description: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub applied_at: Option<String>,
}

// Request types

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SendCoachMessageRequest {
    #[validate(length(min = 1, max = 5000))]
    pub content: String,
    pub image_url: Option<String>,
}

// Response wrappers

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CoachMessagesResponse {
    pub conversation_id: String,
    pub messages: Vec<CoachMessage>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CoachMessageResponse {
    pub message: CoachMessage,
}
