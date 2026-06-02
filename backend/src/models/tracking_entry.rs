use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// A single measurement within a tracking entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Measurement {
    pub metric_id: Uuid,
    pub value: serde_json::Value,
}

/// A tracking entry is a timestamped event on a plant that can combine any of:
/// - Care task completions (which care tasks were performed)
/// - Measurements (metric + value pairs)
/// - Notes (free text)
/// - Photos (attached images)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrackingEntry {
    pub id: Uuid,
    pub plant_id: Uuid,
    pub timestamp: DateTime<Utc>,
    /// Care tasks completed in this entry
    pub care_task_ids: Option<Vec<Uuid>>,
    /// Measurements recorded in this entry
    pub measurements: Option<Vec<Measurement>>,
    pub notes: Option<String>,
    /// Photo IDs attached to this entry
    pub photo_ids: Option<Vec<Uuid>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateTrackingEntryRequest {
    pub timestamp: DateTime<Utc>,
    /// Care tasks to mark as completed
    pub care_task_ids: Option<Vec<Uuid>>,
    /// Measurements to record
    pub measurements: Option<Vec<Measurement>>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
    /// Photo IDs to attach
    pub photo_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTrackingEntryRequest {
    pub timestamp: Option<DateTime<Utc>>,
    pub care_task_ids: Option<Vec<Uuid>>,
    pub measurements: Option<Vec<Measurement>>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
    pub photo_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TrackingEntriesResponse {
    pub entries: Vec<TrackingEntry>,
    pub total: i64,
}
