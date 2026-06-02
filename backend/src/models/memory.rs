use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ─── Fact Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MemoryFactType {
    Location,
    Light,
    Soil,
    Pot,
    WateringPreference,
    Temperature,
    Humidity,
    GrowthHabit,
    SymptomPattern,
    PestHistory,
    FertilizerPreference,
    Propagation,
    Acquisition,
    SpeciesNote,
    General,
}

impl std::fmt::Display for MemoryFactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Location => "location",
            Self::Light => "light",
            Self::Soil => "soil",
            Self::Pot => "pot",
            Self::WateringPreference => "watering_preference",
            Self::Temperature => "temperature",
            Self::Humidity => "humidity",
            Self::GrowthHabit => "growth_habit",
            Self::SymptomPattern => "symptom_pattern",
            Self::PestHistory => "pest_history",
            Self::FertilizerPreference => "fertilizer_preference",
            Self::Propagation => "propagation",
            Self::Acquisition => "acquisition",
            Self::SpeciesNote => "species_note",
            Self::General => "general",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for MemoryFactType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "location" => Ok(Self::Location),
            "light" => Ok(Self::Light),
            "soil" => Ok(Self::Soil),
            "pot" => Ok(Self::Pot),
            "watering_preference" => Ok(Self::WateringPreference),
            "temperature" => Ok(Self::Temperature),
            "humidity" => Ok(Self::Humidity),
            "growth_habit" => Ok(Self::GrowthHabit),
            "symptom_pattern" => Ok(Self::SymptomPattern),
            "pest_history" => Ok(Self::PestHistory),
            "fertilizer_preference" => Ok(Self::FertilizerPreference),
            "propagation" => Ok(Self::Propagation),
            "acquisition" => Ok(Self::Acquisition),
            "species_note" => Ok(Self::SpeciesNote),
            "general" => Ok(Self::General),
            other => Err(format!("Unknown fact type: {other}")),
        }
    }
}

// ─── Memory Source ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MemorySource {
    Coach,
    User,
}

impl std::fmt::Display for MemorySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Coach => write!(f, "coach"),
            Self::User => write!(f, "user"),
        }
    }
}

impl std::str::FromStr for MemorySource {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "coach" => Ok(Self::Coach),
            "user" => Ok(Self::User),
            other => Err(format!("Unknown source: {other}")),
        }
    }
}

// ─── Plant Memory ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlantMemory {
    pub id: Uuid,
    pub plant_id: Uuid,
    pub fact_type: MemoryFactType,
    pub content: String,
    pub confidence: f64,
    pub source: MemorySource,
    pub source_message_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── API Requests ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateMemoryRequest {
    pub fact_type: MemoryFactType,
    pub content: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMemoryRequest {
    pub content: Option<String>,
    pub fact_type: Option<MemoryFactType>,
}

// ─── API Response ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlantMemoriesResponse {
    pub memories: Vec<PlantMemory>,
}

// ─── Coach Extraction (internal, not API-exposed) ────────────────────────────

/// Facts extracted by the coach from a conversation turn.
/// The coach returns these as part of its structured response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedFact {
    pub fact_type: String,
    pub content: String,
    /// 0.0–1.0, how confident the AI is about this fact
    pub confidence: f64,
}

// ─── Health Score ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlantHealthScore {
    pub id: Uuid,
    pub plant_id: Uuid,
    /// 0.0–1.0, displayed as 0–5 hearts
    pub score: f64,
    pub care_adherence: Option<f64>,
    pub overdue_penalty: Option<f64>,
    pub coach_sentiment: Option<f64>,
    pub scored_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Number of hearts (0–5, supports halves like 3.5)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HealthHearts {
    /// 0.0–5.0, rounded to nearest 0.5
    pub hearts: f64,
    pub score: f64,
    pub scored_at: Option<DateTime<Utc>>,
}

impl PlantHealthScore {
    pub fn to_hearts(&self) -> HealthHearts {
        // Map 0.0–1.0 to 0–5 hearts, rounded to nearest 0.5
        let raw = self.score * 5.0;
        let hearts = (raw * 2.0).round() / 2.0;
        HealthHearts {
            hearts,
            score: self.score,
            scored_at: Some(self.scored_at),
        }
    }
}
