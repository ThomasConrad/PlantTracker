use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TrackingEntry {
    pub id: Uuid,
    pub plant_id: Uuid,
    pub entry_type: EntryType,
    pub timestamp: DateTime<Utc>,
    pub value: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub metric_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "entry_type", rename_all = "lowercase")]
pub enum EntryType {
    Watering,
    Fertilizing,
    CustomMetric,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTrackingEntryRequest {
    pub entry_type: EntryType,
    pub timestamp: DateTime<Utc>,
    pub value: Option<serde_json::Value>,
    #[validate(length(max = 1000))]
    pub notes: Option<String>,
    pub metric_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTrackingEntryRequest {
    pub timestamp: Option<DateTime<Utc>>,
    pub value: Option<serde_json::Value>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TrackingEntriesResponse {
    pub entries: Vec<TrackingEntry>,
    pub total: i64,
}
