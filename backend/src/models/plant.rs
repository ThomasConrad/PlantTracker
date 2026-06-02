use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::models::care_task::CareTaskWithStatus;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Plant {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub genus: String,
    pub preview_id: Option<Uuid>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomMetric {
    pub id: Uuid,
    pub plant_id: Uuid,
    pub name: String,
    pub unit: String,
    pub data_type: MetricDataType,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "metric_data_type", rename_all = "lowercase")]
pub enum MetricDataType {
    Number,
    Text,
    Boolean,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlantRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 100))]
    pub genus: String,
    pub custom_metrics: Option<Vec<CreateCustomMetricRequest>>,
    /// Initial care tasks to create with the plant
    #[validate(nested)]
    pub care_tasks: Option<Vec<CreatePlantCareTaskInput>>,
}

/// Inline care task definition used when creating a plant
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlantCareTaskInput {
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
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct CreateCustomMetricRequest {
    #[validate(length(min = 1, max = 50))]
    pub name: String,
    #[validate(length(max = 20))]
    pub unit: String,
    pub data_type: MetricDataType,
}

#[derive(Debug, Deserialize, ToSchema)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlantRequest {
    pub name: Option<String>,
    pub genus: Option<String>,
    pub custom_metrics: Option<Vec<UpdateCustomMetricRequest>>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct UpdateCustomMetricRequest {
    pub id: Option<Uuid>,
    pub name: String,
    pub unit: String,
    pub data_type: MetricDataType,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlantResponse {
    pub id: Uuid,
    pub name: String,
    pub genus: String,
    pub preview_id: Option<Uuid>,
    pub preview_url: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub custom_metrics: Vec<CustomMetric>,
    pub care_tasks: Vec<CareTaskWithStatus>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlantsResponse {
    pub plants: Vec<PlantResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
