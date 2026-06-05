//! Plant identification endpoint.
//!
//! Takes a photo (base64), sends it to OpenAI vision for identification,
//! then enriches the top candidates with data from Trefle.io.

use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use utoipa::ToSchema;

use crate::app_state::AppState;
use crate::extractors::AuthenticatedUser;
use crate::llm::{ChatMessage, ContentPart, ImageUrlContent};
use crate::trefle::TrefleClient;
use crate::utils::errors::{AppError, Result};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/identify", post(identify_plant))
        .route("/species-search", get(search_species))
}

// ─── Request / Response Types ───────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct IdentifyPlantRequest {
    /// Base64-encoded image data URL (e.g., "data:image/jpeg;base64,...")
    pub image_url: String,
    /// Optional context from the user (e.g., "indoor plant on my desk")
    pub context: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct IdentifyPlantResponse {
    /// List of candidate species, sorted by confidence (highest first)
    pub candidates: Vec<PlantCandidate>,
    /// Whether the top candidate is confident enough for auto-selection
    pub auto_select: bool,
    /// General observations from the AI about the photo
    pub analysis_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlantCandidate {
    /// Scientific name (genus + species)
    pub scientific_name: String,
    /// Common name (if known)
    pub common_name: Option<String>,
    /// Genus only
    pub genus: String,
    /// AI confidence 0.0-1.0
    pub confidence: f64,
    /// Why the AI thinks this is the species
    pub reasoning: String,
    /// Reference images from Trefle.io (URLs)
    pub reference_images: Vec<String>,
    /// Suggested care tasks based on Trefle growth data + AI suggestions
    pub suggested_care: SuggestedCare,
    /// Trefle species slug (for further lookups)
    pub trefle_slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SuggestedCare {
    /// Suggested watering interval in days
    pub watering_interval_days: Option<i32>,
    /// Suggested fertilizing interval in days
    pub fertilizing_interval_days: Option<i32>,
    /// Light requirement description
    pub light_requirement: Option<String>,
    /// Humidity notes
    pub humidity_notes: Option<String>,
    /// Temperature range description
    pub temperature_notes: Option<String>,
    /// Additional care notes from AI
    pub additional_notes: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchSpeciesQuery {
    /// Search query (common or scientific name)
    pub q: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchSpeciesResponse {
    pub results: Vec<SpeciesSearchResult>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SpeciesSearchResult {
    pub scientific_name: String,
    pub common_name: Option<String>,
    pub genus: Option<String>,
    pub family: Option<String>,
    pub image_url: Option<String>,
    pub slug: String,
}

// ─── AI Response Schema (internal, parsed from OpenAI response) ─────────────

#[derive(Debug, Deserialize)]
struct AiIdentifyResponse {
    candidates: Vec<AiCandidate>,
    #[serde(default)]
    analysis_notes: String,
}

#[derive(Debug, Deserialize)]
struct AiCandidate {
    scientific_name: String,
    #[serde(default)]
    common_name: Option<String>,
    confidence: f64,
    #[serde(default)]
    reasoning: String,
    #[serde(default)]
    care_suggestions: Option<AiCareSuggestions>,
}

#[derive(Debug, Deserialize)]
struct AiCareSuggestions {
    watering_interval_days: Option<i32>,
    fertilizing_interval_days: Option<i32>,
    light_requirement: Option<String>,
    humidity_notes: Option<String>,
    temperature_notes: Option<String>,
    additional_notes: Option<String>,
}

// ─── Handlers ───────────────────────────────────────────────────────────────

/// Identify a plant from a photo using AI vision + Trefle.io enrichment.
#[utoipa::path(
    post,
    path = "/plants/identify",
    request_body = IdentifyPlantRequest,
    responses(
        (status = 200, description = "Plant identification results", body = IdentifyPlantResponse),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Invalid image data"),
        (status = 502, description = "AI service or Trefle.io unavailable"),
    ),
    security(
        ("session" = [])
    ),
    tag = "plants"
)]
pub async fn identify_plant(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    Json(payload): Json<IdentifyPlantRequest>,
) -> Result<Json<IdentifyPlantResponse>> {

    // Validate image data URL
    if !payload.image_url.starts_with("data:image/") {
        return Err(AppError::Parse {
            message: "image_url must be a base64 data URL starting with 'data:image/'".to_string(),
        });
    }

    // Resolve LLM coach: per-user settings take priority
    let coach = user.resolve_coach(app_state.coach.as_ref())?;
    let coach_ref: &dyn crate::llm::PlantCoach = coach.as_ref();

    // Build the identification prompt
    let system_message = ChatMessage {
        role: "system".to_string(),
        content: vec![ContentPart::Text {
            text: IDENTIFY_SYSTEM_PROMPT.to_string(),
        }],
    };

    let mut user_content = vec![ContentPart::ImageUrl {
        image_url: ImageUrlContent {
            url: payload.image_url.clone(),
        },
    }];

    let user_text = if let Some(ref ctx) = payload.context {
        format!(
            "Identify this plant. Additional context from the user: {}",
            ctx
        )
    } else {
        "Identify this plant from this photo.".to_string()
    };
    user_content.push(ContentPart::Text { text: user_text });

    let user_message = ChatMessage {
        role: "user".to_string(),
        content: user_content,
    };

    info!("Plant identification request from user {}", user.id);

    // Call the AI with custom identification schema (non-streaming, need full structured response)
    let raw_response = coach_ref
        .chat_raw(
            vec![system_message, user_message],
            Some(identify_response_schema()),
        )
        .await
        .map_err(|e| {
            error!("Plant identification AI call failed: {}", e);
            AppError::External {
                message: format!("AI identification failed: {e}"),
            }
        })?;

    // Parse the AI response text as our identification schema
    let ai_response: AiIdentifyResponse = parse_ai_identify_response(&raw_response)?;

    debug!(
        "AI returned {} candidates, notes: {:?}",
        ai_response.candidates.len(),
        &ai_response.analysis_notes
    );

    // Enrich candidates with Trefle.io data (best-effort, don't fail if Trefle is down)
    let trefle_client = TrefleClient::from_env().ok();
    let mut candidates = Vec::new();

    for ai_candidate in ai_response.candidates.iter().take(5) {
        let mut candidate = PlantCandidate {
            scientific_name: ai_candidate.scientific_name.clone(),
            common_name: ai_candidate.common_name.clone(),
            genus: extract_genus(&ai_candidate.scientific_name),
            confidence: ai_candidate.confidence,
            reasoning: ai_candidate.reasoning.clone(),
            reference_images: Vec::new(),
            suggested_care: SuggestedCare {
                watering_interval_days: ai_candidate
                    .care_suggestions
                    .as_ref()
                    .and_then(|c| c.watering_interval_days),
                fertilizing_interval_days: ai_candidate
                    .care_suggestions
                    .as_ref()
                    .and_then(|c| c.fertilizing_interval_days),
                light_requirement: ai_candidate
                    .care_suggestions
                    .as_ref()
                    .and_then(|c| c.light_requirement.clone()),
                humidity_notes: ai_candidate
                    .care_suggestions
                    .as_ref()
                    .and_then(|c| c.humidity_notes.clone()),
                temperature_notes: ai_candidate
                    .care_suggestions
                    .as_ref()
                    .and_then(|c| c.temperature_notes.clone()),
                additional_notes: ai_candidate
                    .care_suggestions
                    .as_ref()
                    .and_then(|c| c.additional_notes.clone()),
            },
            trefle_slug: None,
        };

        // Try to enrich from Trefle.io
        if let Some(ref trefle) = trefle_client {
            match trefle.search_and_enrich(&ai_candidate.scientific_name).await {
                Ok(Some(enriched)) => {
                    // Add reference images
                    candidate.reference_images = enriched
                        .image_urls()
                        .into_iter()
                        .take(4)
                        .collect();

                    candidate.trefle_slug = Some(enriched.species.slug.clone());

                    // Use Trefle common name if AI didn't provide one
                    if candidate.common_name.is_none() {
                        candidate.common_name = enriched.species.common_name.clone();
                    }

                    // Merge care data: Trefle provides data-driven values,
                    // AI provides contextual suggestions. Prefer Trefle when available.
                    if let Some(watering) = enriched.suggested_watering_days() {
                        candidate.suggested_care.watering_interval_days = Some(watering);
                    }
                    if let Some(fertilizing) = enriched.suggested_fertilizing_days() {
                        candidate.suggested_care.fertilizing_interval_days = Some(fertilizing);
                    }
                    if let Some(light) = enriched.light_description() {
                        candidate.suggested_care.light_requirement = Some(light);
                    }
                    if let Some(humidity) = enriched.humidity_description() {
                        candidate.suggested_care.humidity_notes = Some(humidity);
                    }

                    // Temperature notes from Trefle
                    if let Some(growth) = enriched.growth() {
                        let min_temp = growth
                            .minimum_temperature
                            .as_ref()
                            .and_then(|t| t.value);
                        let max_temp = growth
                            .maximum_temperature
                            .as_ref()
                            .and_then(|t| t.value);
                        if min_temp.is_some() || max_temp.is_some() {
                            let temp_str = match (min_temp, max_temp) {
                                (Some(min), Some(max)) => {
                                    format!("{}°C to {}°C", min as i32, max as i32)
                                }
                                (Some(min), None) => format!("above {}°C", min as i32),
                                (None, Some(max)) => format!("below {}°C", max as i32),
                                _ => unreachable!(),
                            };
                            candidate.suggested_care.temperature_notes = Some(temp_str);
                        }
                    }
                }
                Ok(None) => {
                    debug!(
                        "No Trefle match for {:?}",
                        ai_candidate.scientific_name
                    );
                }
                Err(e) => {
                    warn!(
                        "Trefle enrichment failed for {:?}: {}",
                        ai_candidate.scientific_name, e
                    );
                }
            }
        }

        candidates.push(candidate);
    }

    // Determine auto-select: top candidate ≥ 85% AND well-separated from #2
    let auto_select = if let Some(top) = candidates.first() {
        let second_confidence = candidates.get(1).map(|c| c.confidence).unwrap_or(0.0);
        top.confidence >= 0.85 && (top.confidence - second_confidence) >= 0.15
    } else {
        false
    };

    Ok(Json(IdentifyPlantResponse {
        candidates,
        auto_select,
        analysis_notes: ai_response.analysis_notes,
    }))
}

/// Search species via Trefle.io (proxy endpoint to keep API token server-side).
#[utoipa::path(
    get,
    path = "/plants/species-search",
    params(
        ("q" = String, Query, description = "Search query")
    ),
    responses(
        (status = 200, description = "Species search results", body = SearchSpeciesResponse),
        (status = 401, description = "Unauthorized"),
        (status = 502, description = "Trefle.io unavailable"),
    ),
    security(
        ("session" = [])
    ),
    tag = "plants"
)]
pub async fn search_species(
    AuthenticatedUser(_user): AuthenticatedUser,
    axum::extract::Query(params): axum::extract::Query<SearchSpeciesQuery>,
) -> Result<Json<SearchSpeciesResponse>> {

    if params.q.trim().is_empty() {
        return Ok(Json(SearchSpeciesResponse { results: vec![] }));
    }

    let trefle = TrefleClient::from_env().map_err(|e| AppError::External {
        message: format!("Plant database not configured: {e}"),
    })?;

    let species = trefle
        .search_species(&params.q, 10)
        .await
        .map_err(|e| AppError::External {
            message: format!("Plant database search failed: {e}"),
        })?;

    let results = species
        .into_iter()
        .map(|s| SpeciesSearchResult {
            scientific_name: s.scientific_name,
            common_name: s.common_name,
            genus: s.genus,
            family: s.family,
            image_url: s.image_url,
            slug: s.slug,
        })
        .collect();

    Ok(Json(SearchSpeciesResponse { results }))
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Parse AI response text into our identification schema.
/// Handles markdown code fences and lenient JSON parsing.
fn parse_ai_identify_response(text: &str) -> Result<AiIdentifyResponse> {
    let text = text.trim();

    // Strip markdown code fences if present
    let text = text
        .strip_prefix("```json")
        .or_else(|| text.strip_prefix("```"))
        .unwrap_or(text);
    let text = text.strip_suffix("```").unwrap_or(text).trim();

    serde_json::from_str(text).map_err(|e| {
        error!(
            "Failed to parse AI identify response. Error: {}. Raw text ({} bytes): {}",
            e,
            text.len(),
            &text[..text.len().min(500)]
        );
        AppError::External {
            message: format!("AI returned invalid identification response: {e}"),
        }
    })
}

/// Extract genus from scientific name (first word).
fn extract_genus(scientific_name: &str) -> String {
    scientific_name
        .split_whitespace()
        .next()
        .unwrap_or(scientific_name)
        .to_string()
}

// ─── System Prompt ──────────────────────────────────────────────────────────

const IDENTIFY_SYSTEM_PROMPT: &str = r#"You are an expert botanist and plant identification specialist. Analyze the provided photo and identify the plant species.

RESPONSE FORMAT — respond with a single JSON object:
{
  "candidates": [
    {
      "scientific_name": "Genus species",
      "common_name": "Common Name",
      "confidence": 0.92,
      "reasoning": "Brief explanation of identifying features you observe",
      "care_suggestions": {
        "watering_interval_days": 7,
        "fertilizing_interval_days": 14,
        "light_requirement": "bright indirect light",
        "humidity_notes": "prefers moderate to high humidity",
        "temperature_notes": "18-27°C",
        "additional_notes": "any other relevant care info"
      }
    }
  ],
  "analysis_notes": "Overall description of what you observe in the photo"
}

RULES:
- confidence is a float 0.0 to 1.0. Be honest and calibrated — 0.95+ only when you are very certain.
- Include 1-5 candidates, sorted by confidence (highest first).
- Only include candidates with confidence > 0.1.
- If the photo is unclear, blurry, or doesn't show a plant, say so in analysis_notes and return empty candidates.
- scientific_name should use proper binomial nomenclature (Genus species).
- common_name should be the most widely recognized English common name.
- care_suggestions should reflect typical indoor care for that species.
- watering_interval_days: typical days between waterings for indoor plants.
- For "reasoning", mention specific visual features: leaf shape, variegation, growth pattern, stem structure, flower type, etc.
- Respond ONLY with the JSON object. No markdown fences, no extra text outside the JSON.
"#;

/// JSON schema for the plant identification structured output.
fn identify_response_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "candidates": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "scientific_name": { "type": "string" },
                        "common_name": { "type": "string" },
                        "confidence": { "type": "number" },
                        "reasoning": { "type": "string" },
                        "care_suggestions": {
                            "type": "object",
                            "properties": {
                                "watering_interval_days": { "type": "integer" },
                                "fertilizing_interval_days": { "type": "integer" },
                                "light_requirement": { "type": "string" },
                                "humidity_notes": { "type": "string" },
                                "temperature_notes": { "type": "string" },
                                "additional_notes": { "type": "string" }
                            }
                        }
                    },
                    "required": ["scientific_name", "confidence", "reasoning"]
                }
            },
            "analysis_notes": { "type": "string" }
        },
        "required": ["candidates", "analysis_notes"]
    })
}
