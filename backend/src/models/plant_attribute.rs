use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

// ─── Source ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AttributeSource {
    Identification,
    Coach,
    User,
}

impl std::fmt::Display for AttributeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identification => write!(f, "identification"),
            Self::Coach => write!(f, "coach"),
            Self::User => write!(f, "user"),
        }
    }
}

impl std::str::FromStr for AttributeSource {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "identification" => Ok(Self::Identification),
            "coach" => Ok(Self::Coach),
            "user" => Ok(Self::User),
            other => Err(format!("Unknown attribute source: {other}")),
        }
    }
}

// ─── Database Row ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct PlantAttributeRow {
    pub id: String,
    pub plant_id: String,
    pub user_id: String,
    pub key: String,
    pub label: String,
    pub value: String,
    pub icon: Option<String>,
    pub category: Option<String>,
    pub source: String,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

// ─── API Response ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlantAttribute {
    pub id: Uuid,
    pub plant_id: Uuid,
    pub key: String,
    pub label: String,
    pub value: String,
    pub icon: Option<String>,
    pub category: Option<String>,
    pub source: AttributeSource,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlantAttributesResponse {
    pub attributes: Vec<PlantAttribute>,
}

// ─── API Requests ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlantAttributeRequest {
    #[validate(length(min = 1, max = 50))]
    pub key: String,
    #[validate(length(min = 1, max = 100))]
    pub label: String,
    #[validate(length(min = 1, max = 1000))]
    pub value: String,
    #[validate(length(max = 10))]
    pub icon: Option<String>,
    #[validate(length(max = 50))]
    pub category: Option<String>,
    pub source: Option<AttributeSource>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlantAttributeRequest {
    #[validate(length(min = 1, max = 100))]
    pub label: Option<String>,
    #[validate(length(min = 1, max = 1000))]
    pub value: Option<String>,
    #[validate(length(max = 10))]
    pub icon: Option<Option<String>>,
    #[validate(length(max = 50))]
    pub category: Option<Option<String>>,
}

// ─── Conversion ──────────────────────────────────────────────────────────────

impl PlantAttributeRow {
    pub fn into_response(self) -> PlantAttribute {
        PlantAttribute {
            id: self.id.parse().unwrap_or_default(),
            plant_id: self.plant_id.parse().unwrap_or_default(),
            key: self.key,
            label: self.label,
            value: self.value,
            icon: self.icon,
            category: self.category,
            source: self.source.parse().unwrap_or(AttributeSource::User),
            sort_order: self.sort_order as i32,
            created_at: self
                .created_at
                .parse::<DateTime<Utc>>()
                .unwrap_or_default(),
            updated_at: self
                .updated_at
                .parse::<DateTime<Utc>>()
                .unwrap_or_default(),
        }
    }
}
