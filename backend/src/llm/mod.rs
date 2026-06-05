pub mod anthropic;
pub mod mock;
pub mod ollama;
pub mod openai;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "system", "user", "assistant"
    pub content: Vec<ContentPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrlContent },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrlContent {
    pub url: String, // base64 data URL or https URL
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachResponse {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub suggestions: Vec<CoachSuggestionOutput>,
    /// Facts extracted from the conversation to store as plant memories
    #[serde(default)]
    pub extracted_facts: Vec<ExtractedFactOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachSuggestionOutput {
    #[serde(default)]
    pub suggestion_type: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFactOutput {
    /// One of: location, light, soil, pot, watering_preference, temperature, humidity,
    /// growth_habit, symptom_pattern, pest_history, fertilizer_preference, propagation,
    /// acquisition, species_note, general
    pub fact_type: String,
    pub content: String,
    /// 0.0–1.0
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

fn default_confidence() -> f64 {
    0.7
}

#[async_trait::async_trait]
pub trait PlantCoach: Send + Sync {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<CoachResponse>;

    /// Chat with a custom JSON schema for structured output.
    /// Returns the raw text content (caller is responsible for parsing).
    async fn chat_raw(
        &self,
        messages: Vec<ChatMessage>,
        _response_schema: Option<serde_json::Value>,
    ) -> Result<String> {
        // Default: use regular chat and return the text field
        let response = self.chat(messages).await?;
        Ok(response.text)
    }

    /// Stream tokens through the sender, then return the final parsed response.
    /// Default implementation falls back to non-streaming chat.
    async fn stream_chat(
        &self,
        messages: Vec<ChatMessage>,
        tx: tokio::sync::mpsc::Sender<String>,
    ) -> Result<CoachResponse> {
        let response = self.chat(messages).await?;
        // Send the text as a single chunk
        let _ = tx.send(response.text.clone()).await;
        Ok(response)
    }
}

/// Factory: create the coach from environment config
pub fn create_coach() -> Result<Box<dyn PlantCoach>> {
    let provider = std::env::var("PLANT_COACH_PROVIDER").unwrap_or_else(|_| "openai".to_string());
    match provider.as_str() {
        "anthropic" => Ok(Box::new(anthropic::AnthropicCoach::new()?)),
        "ollama" => Ok(Box::new(ollama::OllamaCoach::new()?)),
        "mock" => Ok(Box::new(mock::MockCoach)),
        _ => Ok(Box::new(openai::OpenAICoach::new()?)),
    }
}

/// Create a coach from per-user LLM settings (OpenAI-compatible endpoint).
/// Falls back to the global coach if user has no settings configured.
pub fn create_coach_for_user(
    base_url: Option<&str>,
    api_key: Option<&str>,
    model: Option<&str>,
) -> Option<Result<Box<dyn PlantCoach>>> {
    // Need at least a base_url to create a user-specific coach
    let base_url = base_url?;
    if base_url.is_empty() {
        return None;
    }
    Some(
        openai::OpenAICoach::new_with_config(
            base_url.to_string(),
            api_key.unwrap_or("").to_string(),
            model.unwrap_or("gpt-4o").to_string(),
        )
        .map(|c| Box::new(c) as Box<dyn PlantCoach>),
    )
}

// ─── Shared prompt & schema ─────────────────────────────────────────────────

/// The base system prompt instructing the LLM how to behave and what JSON to return.
/// Plant-specific context is appended by `build_plant_context()` before sending.
pub const COACH_SYSTEM_PROMPT: &str = r#"You are an expert plant care coach with deep horticultural knowledge. You help users care for their plants by analyzing photos, diagnosing problems, and suggesting care adjustments.

EXPERTISE — draw on this knowledge when relevant:
- Light: understand PAR/DLI, footcandles, orientation (N/S/E/W windows), seasonal changes, etiolation signs
- Water: soil moisture cues (weight, finger test, meter), drainage importance, water quality (chlorine, fluoride sensitivity), bottom watering vs top
- Soil: aroid mixes, succulent mixes, perlite/pumice/bark ratios, pH preferences, compaction over time
- Humidity: tropical vs arid species, grouping, pebble trays, humidifiers, crispy tips vs root rot tradeoffs
- Fertilizing: NPK ratios, micro-nutrients, salt buildup/flushing, seasonal feeding (reduce in winter dormancy)
- Pests: spider mites (webbing, stippling), thrips (silver streaks), mealybugs (cottony masses), fungus gnats (larvae in soil), scale, aphids — treatment options (neem, systemic, isopropyl, beneficial insects)
- Disease: root rot (mushy stems, yellowing), powdery mildew, bacterial leaf spot, viral mosaic
- Propagation: stem cuttings, water vs soil, air layering, division, leaf cuttings (succulents/begonias)
- Seasonal: dormancy periods, growth seasons, light/water adjustments, hardening off
- Stress signs: leaf curling, drooping, yellowing (overwater vs underwater vs nutrient), brown tips, leggy growth

COMMUNICATION STYLE:
- Adapt your detail level to the user's apparent experience. If they use technical terms (nodes, fenestrations, substrate), respond concisely and technically. If they ask basic questions, explain gently with context.
- Be specific: say "water when the top 2 inches are dry" not "water when needed"
- When diagnosing from photos: describe exactly what you observe, list possible causes ranked by likelihood, and suggest one clear next step
- Celebrate progress ("the new growth looks great!") — plant care should feel rewarding
- If you're uncertain, say so and ask a clarifying question rather than guessing

RESPONSE FORMAT — always respond with a single JSON object:
{
  "text": "Your conversational response (markdown OK)",
  "suggestions": []
}

SUGGESTION TYPES — include only when the conversation warrants actionable changes:

1. "schedule_change" — adjust an existing care task's interval
   payload: { "careTaskName": "<exact task name>", "intervalDays": <number> }
   Use when: user reports overwatering/underwatering, seasonal shift, plant moved to new light

2. "new_task" — create a brand-new care task
   payload: { "name": "<task name>", "icon": "<single emoji>", "intervalDays": <number> }
   Use when: suggesting misting, rotating, flushing soil, checking roots — anything not already tracked

3. "care_action" — suggest the user perform a specific task right now
   payload: { "careTaskName": "<exact task name>" }
   Use when: plant shows immediate need (wilting → water now, pests → treat now)

4. "photo_request" — ask the user to take/upload a photo for diagnosis
   payload: {}
   Use when: you need visual information to diagnose, or to track progress over time

5. "species_correction" — correct the plant's recorded species/genus
   payload: { "genus": "<correct genus>", "species": "<correct species if known>" }
   Use when: from photos or description you're confident the recorded species is wrong

MEMORY EXTRACTION — extract facts the user reveals (or you observe from photos) about the plant's environment, care, and behavior. Include in "extracted_facts" array:
{
  "extracted_facts": [
    { "fact_type": "<type>", "content": "<concise fact>", "confidence": 0.0-1.0 }
  ]
}

Fact types: location, light, soil, pot, watering_preference, temperature, humidity, growth_habit, symptom_pattern, pest_history, fertilizer_preference, propagation, acquisition, species_note, general

Guidelines for extraction:
- Extract when the user mentions WHERE the plant is, WHAT soil/pot it's in, HOW they water, etc.
- From photos: extract observable facts (pot type, light level, growth stage, visible issues)
- Use confidence 0.9+ for facts the user directly states ("it's on my south window")
- Use confidence 0.6-0.8 for facts you infer ("looks like bright indirect light based on the photo")
- DON'T re-extract facts that are already in "Known Facts" unless updating them with new info
- Keep content concise: "south-facing kitchen windowsill" not "The plant is placed on a windowsill that faces south in the user's kitchen"

RULES:
- Respond ONLY with the JSON object. No markdown fences, no extra text outside the JSON.
- Use careTaskName that exactly matches one of the plant's existing care tasks (listed in context).
- Keep suggestions practical and specific to the plant's species and current state.
- Consider the genus when advising — a succulent and a fern have opposite needs.
- Factor in the care history: if a task was last performed recently, don't suggest doing it again unless there's a specific reason.
- PHOTO ANALYSIS: When the user sends a photo, describe what you observe in detail (leaf color, texture, size, spots, soil condition, pot, light levels). This description becomes the persistent record — the image won't be re-sent in future messages, so your text analysis must be thorough enough to reference later.
"#;

/// Build plant-specific context to append to the system prompt.
/// Includes care history signals so the LLM can gauge user experience level.
pub fn build_plant_context(plant: &crate::models::PlantResponse) -> String {
    let care_tasks_info = if plant.care_tasks.is_empty() {
        "  (none configured — user hasn't set up care tasks yet, may be new)".to_string()
    } else {
        plant
            .care_tasks
            .iter()
            .map(|t| {
                let schedule = t
                    .task
                    .interval_days
                    .map(|d| format!("every {}d", d))
                    .unwrap_or_else(|| "manual".to_string());
                let last = t
                    .task
                    .last_performed
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "never".to_string());
                let status = if t.is_due {
                    format!(" ⚠️ OVERDUE {}d", t.days_overdue.unwrap_or(0))
                } else if let Some(days) = t.days_overdue {
                    format!(" (due in {}d)", -days)
                } else {
                    String::new()
                };
                format!(
                    "  - {} {} | {} | last: {}{}",
                    t.task.icon.as_deref().unwrap_or("📋"),
                    t.task.name,
                    schedule,
                    last,
                    status,
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let metrics_info = if plant.custom_metrics.is_empty() {
        String::new()
    } else {
        let metrics = plant
            .custom_metrics
            .iter()
            .map(|m| format!("  - {} ({})", m.name, m.unit))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "\nCustom metrics (user tracks these — indicates experience):\n{}",
            metrics
        )
    };

    // Experience signals for the LLM
    let task_count = plant.care_tasks.len();
    let has_custom_metrics = !plant.custom_metrics.is_empty();
    let experience_hint = if task_count > 3 || has_custom_metrics {
        "\nUser experience: likely intermediate/advanced (multiple tasks, custom metrics). Be concise and technical."
    } else if task_count == 0 {
        "\nUser experience: likely beginner (no tasks configured). Explain concepts, be encouraging."
    } else {
        ""
    };

    format!(
        "\n\nPLANT CONTEXT:\nName: {}\nGenus: {}\nCare tasks:\n{}{}{}",
        plant.name, plant.genus, care_tasks_info, metrics_info, experience_hint
    )
}

/// JSON schema for the coach response structured output.
pub fn coach_response_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "text": { "type": "string" },
            "suggestions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "suggestion_type": {
                            "type": "string",
                            "enum": ["schedule_change", "new_task", "care_action", "photo_request", "species_correction"]
                        },
                        "description": { "type": "string" },
                        "payload": { "type": "object" }
                    },
                    "required": ["suggestion_type", "description", "payload"]
                }
            },
            "extracted_facts": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "fact_type": {
                            "type": "string",
                            "enum": [
                                "location", "light", "soil", "pot", "watering_preference",
                                "temperature", "humidity", "growth_habit", "symptom_pattern",
                                "pest_history", "fertilizer_preference", "propagation",
                                "acquisition", "species_note", "general"
                            ]
                        },
                        "content": { "type": "string" },
                        "confidence": { "type": "number", "minimum": 0, "maximum": 1 }
                    },
                    "required": ["fact_type", "content", "confidence"]
                }
            }
        },
        "required": ["text", "suggestions", "extracted_facts"]
    })
}
