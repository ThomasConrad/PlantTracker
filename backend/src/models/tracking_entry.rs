use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrackingEntry {
    pub id: Uuid,
    pub plant_id: Uuid,
    pub entry_type: EntryType,
    pub timestamp: DateTime<Utc>,
    pub value: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub metric_id: Option<Uuid>,
    pub photo_ids: Option<serde_json::Value>, // Array of photo UUIDs
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum EntryType {
    Watering,
    Fertilizing,
    CustomMetric,
    Note,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateTrackingEntryRequest {
    pub entry_type: EntryType,
    pub timestamp: DateTime<Utc>,
    pub value: Option<serde_json::Value>,
    #[validate(length(max = 1000))]
    pub notes: Option<String>,
    pub metric_id: Option<Uuid>,
    pub photo_ids: Option<Vec<Uuid>>, // Array of photo UUIDs
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTrackingEntryRequest {
    pub timestamp: Option<DateTime<Utc>>,
    pub value: Option<serde_json::Value>,
    #[validate(length(max = 1000))]
    pub notes: Option<String>,
    pub photo_ids: Option<Vec<Uuid>>, // Array of photo UUIDs
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TrackingEntriesResponse {
    pub entries: Vec<TrackingEntry>,
    pub total: i64,
}
