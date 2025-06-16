use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CareSchedule {
    pub interval_days: Option<i32>,
    pub amount: Option<f64>,
    pub unit: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateCareScheduleRequest {
    #[validate(range(min = 1, max = 365))]
    pub interval_days: Option<i32>,
    #[validate(range(min = 0.01))]
    pub amount: Option<f64>,
    #[validate(length(max = 20))]
    pub unit: Option<String>,
    #[validate(length(max = 500))]
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCareScheduleRequest {
    pub interval_days: Option<i32>,
    pub amount: Option<f64>,
    pub unit: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Plant {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub genus: String,
    pub watering_interval_days: Option<i32>,
    pub fertilizing_interval_days: Option<i32>,
    pub watering_amount: Option<f64>,
    pub watering_unit: Option<String>,
    pub watering_notes: Option<String>,
    pub fertilizing_amount: Option<f64>,
    pub fertilizing_unit: Option<String>,
    pub fertilizing_notes: Option<String>,
    pub last_watered: Option<DateTime<Utc>>,
    pub last_fertilized: Option<DateTime<Utc>>,
    pub preview_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Plant {
    pub fn watering_schedule(&self) -> CareSchedule {
        CareSchedule {
            interval_days: self.watering_interval_days,
            amount: self.watering_amount,
            unit: self.watering_unit.clone(),
            notes: self.watering_notes.clone(),
        }
    }

    pub fn fertilizing_schedule(&self) -> CareSchedule {
        CareSchedule {
            interval_days: self.fertilizing_interval_days,
            amount: self.fertilizing_amount,
            unit: self.fertilizing_unit.clone(),
            notes: self.fertilizing_notes.clone(),
        }
    }
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
    #[validate(nested)]
    pub watering_schedule: Option<CreateCareScheduleRequest>,
    #[validate(nested)]
    pub fertilizing_schedule: Option<CreateCareScheduleRequest>,
    pub custom_metrics: Option<Vec<CreateCustomMetricRequest>>,
    pub last_watered: Option<DateTime<Utc>>,
    pub last_fertilized: Option<DateTime<Utc>>,
}

impl CreatePlantRequest {
    pub fn watering_interval_days(&self) -> Option<i32> {
        self.watering_schedule
            .as_ref()
            .and_then(|s| s.interval_days)
    }

    pub fn watering_amount(&self) -> Option<f64> {
        self.watering_schedule.as_ref().and_then(|s| s.amount)
    }

    pub fn watering_unit(&self) -> Option<String> {
        self.watering_schedule.as_ref().and_then(|s| s.unit.clone())
    }

    pub fn watering_notes(&self) -> Option<String> {
        self.watering_schedule
            .as_ref()
            .and_then(|s| s.notes.clone())
    }

    pub fn fertilizing_interval_days(&self) -> Option<i32> {
        self.fertilizing_schedule
            .as_ref()
            .and_then(|s| s.interval_days)
    }

    pub fn fertilizing_amount(&self) -> Option<f64> {
        self.fertilizing_schedule.as_ref().and_then(|s| s.amount)
    }

    pub fn fertilizing_unit(&self) -> Option<String> {
        self.fertilizing_schedule
            .as_ref()
            .and_then(|s| s.unit.clone())
    }

    pub fn fertilizing_notes(&self) -> Option<String> {
        self.fertilizing_schedule
            .as_ref()
            .and_then(|s| s.notes.clone())
    }
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
    pub watering_schedule: Option<UpdateCareScheduleRequest>,
    pub fertilizing_schedule: Option<UpdateCareScheduleRequest>,
    pub custom_metrics: Option<Vec<UpdateCustomMetricRequest>>,
}

impl UpdatePlantRequest {
    pub fn watering_interval_days(&self) -> Option<Option<i32>> {
        self.watering_schedule.as_ref().map(|s| s.interval_days)
    }

    pub fn watering_amount(&self) -> Option<Option<f64>> {
        self.watering_schedule.as_ref().map(|s| s.amount)
    }

    pub fn watering_unit(&self) -> Option<Option<String>> {
        self.watering_schedule.as_ref().map(|s| s.unit.clone())
    }

    pub fn watering_notes(&self) -> Option<Option<String>> {
        self.watering_schedule.as_ref().map(|s| s.notes.clone())
    }

    pub fn fertilizing_interval_days(&self) -> Option<Option<i32>> {
        self.fertilizing_schedule.as_ref().map(|s| s.interval_days)
    }

    pub fn fertilizing_amount(&self) -> Option<Option<f64>> {
        self.fertilizing_schedule.as_ref().map(|s| s.amount)
    }

    pub fn fertilizing_unit(&self) -> Option<Option<String>> {
        self.fertilizing_schedule.as_ref().map(|s| s.unit.clone())
    }

    pub fn fertilizing_notes(&self) -> Option<Option<String>> {
        self.fertilizing_schedule.as_ref().map(|s| s.notes.clone())
    }
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
    pub watering_schedule: CareSchedule,
    pub fertilizing_schedule: CareSchedule,
    pub last_watered: Option<DateTime<Utc>>,
    pub last_fertilized: Option<DateTime<Utc>>,
    pub preview_id: Option<Uuid>,
    pub preview_url: Option<String>,
    pub custom_metrics: Vec<CustomMetric>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_create_plant_request_validation_valid() {
        let request = CreatePlantRequest {
            name: "Fiddle Leaf Fig".to_string(),
            genus: "Ficus".to_string(),
            watering_schedule: Some(CreateCareScheduleRequest {
                interval_days: Some(7),
                amount: None,
                unit: None,
                notes: None,
            }),
            fertilizing_schedule: Some(CreateCareScheduleRequest {
                interval_days: Some(14),
                amount: None,
                unit: None,
                notes: None,
            }),
            custom_metrics: None,
            last_watered: None,
            last_fertilized: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_plant_request_validation_empty_name() {
        let request = CreatePlantRequest {
            name: "".to_string(),
            genus: "Ficus".to_string(),
            watering_schedule: Some(CreateCareScheduleRequest {
                interval_days: Some(7),
                amount: None,
                unit: None,
                notes: None,
            }),
            fertilizing_schedule: Some(CreateCareScheduleRequest {
                interval_days: Some(14),
                amount: None,
                unit: None,
                notes: None,
            }),
            custom_metrics: None,
            last_watered: None,
            last_fertilized: None,
        };

        let validation_result = request.validate();
        assert!(validation_result.is_err());

        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("name"));
    }

    #[test]
    fn test_create_plant_request_validation_long_name() {
        let request = CreatePlantRequest {
            name: "a".repeat(101), // Exceeds max length of 100
            genus: "Ficus".to_string(),
            watering_schedule: Some(CreateCareScheduleRequest {
                interval_days: Some(7),
                amount: None,
                unit: None,
                notes: None,
            }),
            fertilizing_schedule: Some(CreateCareScheduleRequest {
                interval_days: Some(14),
                amount: None,
                unit: None,
                notes: None,
            }),
            custom_metrics: None,
            last_watered: None,
            last_fertilized: None,
        };

        let validation_result = request.validate();
        assert!(validation_result.is_err());

        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("name"));
    }

    #[test]
    fn test_create_plant_request_validation_empty_genus() {
        let request = CreatePlantRequest {
            name: "Fiddle Leaf Fig".to_string(),
            genus: "".to_string(),
            watering_schedule: Some(CreateCareScheduleRequest {
                interval_days: Some(7),
                amount: None,
                unit: None,
                notes: None,
            }),
            fertilizing_schedule: Some(CreateCareScheduleRequest {
                interval_days: Some(14),
                amount: None,
                unit: None,
                notes: None,
            }),
            custom_metrics: None,
            last_watered: None,
            last_fertilized: None,
        };

        let validation_result = request.validate();
        assert!(validation_result.is_err());

        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("genus"));
    }

    #[test]
    fn test_create_plant_request_validation_invalid_watering_interval() {
        let request = CreatePlantRequest {
            name: "Fiddle Leaf Fig".to_string(),
            genus: "Ficus".to_string(),
            watering_schedule: Some(CreateCareScheduleRequest {
                interval_days: Some(0), // Below minimum of 1
                amount: None,
                unit: None,
                notes: None,
            }),
            fertilizing_schedule: Some(CreateCareScheduleRequest {
                interval_days: Some(14),
                amount: None,
                unit: None,
                notes: None,
            }),
            custom_metrics: None,
            last_watered: None,
            last_fertilized: None,
        };

        let validation_result = request.validate();
        assert!(validation_result.is_err());

        let errors = validation_result.unwrap_err();
        // Check for nested validation errors in watering_schedule
        assert!(errors.errors().contains_key("watering_schedule"));
    }

    #[test]
    fn test_create_plant_request_validation_invalid_fertilizing_interval() {
        let request = CreatePlantRequest {
            name: "Fiddle Leaf Fig".to_string(),
            genus: "Ficus".to_string(),
            watering_schedule: Some(CreateCareScheduleRequest {
                interval_days: Some(7),
                amount: None,
                unit: None,
                notes: None,
            }),
            fertilizing_schedule: Some(CreateCareScheduleRequest {
                interval_days: Some(366), // Above maximum of 365
                amount: None,
                unit: None,
                notes: None,
            }),
            custom_metrics: None,
            last_watered: None,
            last_fertilized: None,
        };

        let validation_result = request.validate();
        assert!(validation_result.is_err());

        let errors = validation_result.unwrap_err();
        // Check for nested validation errors in fertilizing_schedule
        assert!(errors.errors().contains_key("fertilizing_schedule"));
    }

    #[test]
    fn test_create_custom_metric_request_validation_valid() {
        let request = CreateCustomMetricRequest {
            name: "Height".to_string(),
            unit: "cm".to_string(),
            data_type: MetricDataType::Number,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_custom_metric_request_validation_empty_name() {
        let request = CreateCustomMetricRequest {
            name: "".to_string(),
            unit: "cm".to_string(),
            data_type: MetricDataType::Number,
        };

        let validation_result = request.validate();
        assert!(validation_result.is_err());

        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("name"));
    }

    #[test]
    fn test_create_custom_metric_request_validation_long_unit() {
        let request = CreateCustomMetricRequest {
            name: "Height".to_string(),
            unit: "a".repeat(21), // Exceeds max length of 20
            data_type: MetricDataType::Number,
        };

        let validation_result = request.validate();
        assert!(validation_result.is_err());

        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("unit"));
    }

    #[test]
    fn test_metric_data_type_serialization() {
        let number_type = MetricDataType::Number;
        let text_type = MetricDataType::Text;
        let boolean_type = MetricDataType::Boolean;

        // Test serialization
        let number_json = serde_json::to_string(&number_type).unwrap();
        let text_json = serde_json::to_string(&text_type).unwrap();
        let boolean_json = serde_json::to_string(&boolean_type).unwrap();

        assert_eq!(number_json, "\"Number\"");
        assert_eq!(text_json, "\"Text\"");
        assert_eq!(boolean_json, "\"Boolean\"");

        // Test deserialization
        let deserialized_number: MetricDataType = serde_json::from_str(&number_json).unwrap();
        let deserialized_text: MetricDataType = serde_json::from_str(&text_json).unwrap();
        let deserialized_boolean: MetricDataType = serde_json::from_str(&boolean_json).unwrap();

        assert!(matches!(deserialized_number, MetricDataType::Number));
        assert!(matches!(deserialized_text, MetricDataType::Text));
        assert!(matches!(deserialized_boolean, MetricDataType::Boolean));
    }

    #[test]
    fn test_create_plant_request_with_custom_metrics() {
        let custom_metric = CreateCustomMetricRequest {
            name: "Height".to_string(),
            unit: "cm".to_string(),
            data_type: MetricDataType::Number,
        };

        let request = CreatePlantRequest {
            name: "Fiddle Leaf Fig".to_string(),
            genus: "Ficus".to_string(),
            watering_schedule: Some(CreateCareScheduleRequest {
                interval_days: Some(7),
                amount: Some(250.0),
                unit: Some("ml".to_string()),
                notes: Some("Water when soil is dry".to_string()),
            }),
            fertilizing_schedule: Some(CreateCareScheduleRequest {
                interval_days: Some(14),
                amount: None,
                unit: None,
                notes: None,
            }),
            custom_metrics: Some(vec![custom_metric]),
            last_watered: None,
            last_fertilized: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_update_plant_request_deserialization() {
        let json = r#"{
            "name": "Updated Plant Name",
            "wateringSchedule": {
                "intervalDays": 5
            },
            "fertilizingSchedule": {
                "intervalDays": 21
            }
        }"#;

        let request: UpdatePlantRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.name, Some("Updated Plant Name".to_string()));
        assert_eq!(request.genus, None);
        assert_eq!(request.watering_interval_days(), Some(Some(5)));
        assert_eq!(request.fertilizing_interval_days(), Some(Some(21)));
        assert!(request.custom_metrics.is_none());
    }

    #[test]
    fn test_plants_response_serialization() {
        let plant_response = PlantResponse {
            id: Uuid::new_v4(),
            name: "Test Plant".to_string(),
            genus: "Test Genus".to_string(),
            watering_schedule: CareSchedule {
                interval_days: Some(7),
                amount: None,
                unit: None,
                notes: None,
            },
            fertilizing_schedule: CareSchedule {
                interval_days: Some(14),
                amount: None,
                unit: None,
                notes: None,
            },
            last_watered: None,
            last_fertilized: None,
            preview_id: None,
            preview_url: None,
            custom_metrics: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            user_id: Uuid::new_v4().to_string(),
        };

        let response = PlantsResponse {
            plants: vec![plant_response],
            total: 1,
            limit: 20,
            offset: 0,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"total\":1"));
        assert!(json.contains("\"limit\":20"));
        assert!(json.contains("\"offset\":0"));
        assert!(json.contains("\"plants\":["));
    }

    #[test]
    fn test_plant_debug_format() {
        let plant = Plant {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Test Plant".to_string(),
            genus: "Test Genus".to_string(),
            watering_interval_days: Some(7),
            fertilizing_interval_days: Some(14),
            watering_amount: None,
            watering_unit: None,
            watering_notes: None,
            fertilizing_amount: None,
            fertilizing_unit: None,
            fertilizing_notes: None,
            last_watered: None,
            last_fertilized: None,
            preview_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let debug_output = format!("{:?}", plant);
        assert!(debug_output.contains("Plant"));
        assert!(debug_output.contains("Test Plant"));
        assert!(debug_output.contains("Test Genus"));
    }

    #[test]
    fn test_custom_metric_clone() {
        let metric = CustomMetric {
            id: Uuid::new_v4(),
            plant_id: Uuid::new_v4(),
            name: "Height".to_string(),
            unit: "cm".to_string(),
            data_type: MetricDataType::Number,
        };

        let cloned_metric = metric.clone();
        assert_eq!(metric.id, cloned_metric.id);
        assert_eq!(metric.name, cloned_metric.name);
        assert_eq!(metric.unit, cloned_metric.unit);
    }
}
