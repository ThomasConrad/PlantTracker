use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// A care task represents a specific type of care activity for a plant.
/// Examples: "Water", "Fertilize", "Prune leaves", "Repot", "Rotate", "Mist", etc.
/// Each can optionally have a recurring schedule (interval_days).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CareTask {
    pub id: Uuid,
    pub plant_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub interval_days: Option<i32>,
    pub amount: Option<f64>,
    pub unit: Option<String>,
    pub notes: Option<String>,
    pub last_performed: Option<DateTime<Utc>>,
    pub sort_order: i32,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Computed view of a care task with due-date info for the frontend
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CareTaskWithStatus {
    #[serde(flatten)]
    pub task: CareTask,
    /// When this task is next due (None if no interval or never performed with no interval)
    pub next_due: Option<DateTime<Utc>>,
    /// How many days overdue (negative = days until due, 0 = due today, positive = overdue)
    pub days_overdue: Option<i64>,
    /// Whether the task is currently due
    pub is_due: bool,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateCareTaskRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(max = 10))]
    pub icon: Option<String>,
    #[validate(length(max = 7))]
    pub color: Option<String>,
    #[validate(range(min = 1, max = 365))]
    pub interval_days: Option<i32>,
    #[validate(range(min = 0.01))]
    pub amount: Option<f64>,
    #[validate(length(max = 20))]
    pub unit: Option<String>,
    #[validate(length(max = 500))]
    pub notes: Option<String>,
    pub last_performed: Option<DateTime<Utc>>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCareTaskRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub icon: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub interval_days: Option<Option<i32>>,
    pub amount: Option<Option<f64>>,
    pub unit: Option<Option<String>>,
    pub notes: Option<Option<String>>,
    pub last_performed: Option<Option<DateTime<Utc>>>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReorderCareTasksRequest {
    /// Ordered list of care task IDs in desired order
    pub task_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CareTasksResponse {
    pub tasks: Vec<CareTaskWithStatus>,
}

/// Request to log a care task completion (creates a tracking entry + updates last_performed)
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct LogCareTaskRequest {
    pub timestamp: Option<DateTime<Utc>>,
    pub value: Option<serde_json::Value>,
    #[validate(length(max = 1000))]
    pub notes: Option<String>,
    pub photo_ids: Option<Vec<Uuid>>,
}
