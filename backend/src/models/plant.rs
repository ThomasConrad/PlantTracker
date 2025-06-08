use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Plant {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub genus: String,
    pub watering_interval_days: i32,
    pub fertilizing_interval_days: i32,
    pub last_watered: Option<DateTime<Utc>>,
    pub last_fertilized: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CustomMetric {
    pub id: Uuid,
    pub plant_id: Uuid,
    pub name: String,
    pub unit: String,
    pub data_type: MetricDataType,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "metric_data_type", rename_all = "lowercase")]
pub enum MetricDataType {
    Number,
    Text,
    Boolean,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePlantRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 100))]
    pub genus: String,
    #[validate(range(min = 1, max = 365))]
    pub watering_interval_days: i32,
    #[validate(range(min = 1, max = 365))]
    pub fertilizing_interval_days: i32,
    pub custom_metrics: Option<Vec<CreateCustomMetricRequest>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCustomMetricRequest {
    #[validate(length(min = 1, max = 50))]
    pub name: String,
    #[validate(length(max = 20))]
    pub unit: String,
    pub data_type: MetricDataType,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePlantRequest {
    pub name: Option<String>,
    pub genus: Option<String>,
    pub watering_interval_days: Option<i32>,
    pub fertilizing_interval_days: Option<i32>,
    pub custom_metrics: Option<Vec<UpdateCustomMetricRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCustomMetricRequest {
    pub id: Option<Uuid>,
    pub name: String,
    pub unit: String,
    pub data_type: MetricDataType,
}

#[derive(Debug, Serialize)]
pub struct PlantResponse {
    pub id: Uuid,
    pub name: String,
    pub genus: String,
    pub watering_interval_days: i32,
    pub fertilizing_interval_days: i32,
    pub last_watered: Option<DateTime<Utc>>,
    pub last_fertilized: Option<DateTime<Utc>>,
    pub custom_metrics: Vec<CustomMetric>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct PlantsResponse {
    pub plants: Vec<PlantResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}