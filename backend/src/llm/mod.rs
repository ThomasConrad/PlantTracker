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
    /// Interactive input widgets to show the user
    #[serde(default)]
    pub input_requests: Vec<InputRequestOutput>,
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
pub struct InputRequestOutput {
    /// Template type: toggle, slider, select, multi_select, photo
    pub template: String,
    /// Template-specific parameters (label, key, options, min/max, etc.)
    pub params: serde_json::Value,
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

/// Resolve the best available coach for a user: per-user settings take priority,
/// then falls back to the app-level global coach. Returns an Arc for cheap cloning
/// into spawned tasks.
///
/// Use this instead of manually matching `create_coach_for_user` + `app_state.coach`
/// in every handler.
pub fn resolve_coach_for_request(
    user_base_url: Option<&str>,
    user_api_key: Option<&str>,
    user_model: Option<&str>,
    global_coach: Option<&std::sync::Arc<dyn PlantCoach>>,
) -> std::result::Result<std::sync::Arc<dyn PlantCoach>, String> {
    // Try per-user coach first
    if let Some(result) = create_coach_for_user(user_base_url, user_api_key, user_model) {
        match result {
            Ok(boxed) => return Ok(std::sync::Arc::from(boxed)),
            Err(e) => {
                return Err(format!("Failed to initialize your LLM settings: {e}"));
            }
        }
    }

    // Fall back to global coach
    match global_coach {
        Some(arc) => Ok(arc.clone()),
        None => Err("No AI service configured. Set up your LLM provider in Settings.".to_string()),
    }
}

// ─── Shared prompt & schema ─────────────────────────────────────────────────

/// The base system prompt instructing the LLM how to behave and what JSON to return.
/// Plant-specific context is appended by `build_plant_context()` before sending.
pub const COACH_SYSTEM_PROMPT: &str = r#"You are a plant care coach. You help users care for their plants.

COMMUNICATION STYLE:
- Be SHORT and conversational. 1-3 sentences max unless the user asks for detail.
- No markdown formatting — no **, no *, no #, no bullet lists. Write plain text.
- Use line breaks to separate thoughts if needed, but keep it minimal.
- Be direct: "Water it now, soil looks dry" not "Based on my analysis of the photo, I would recommend watering..."
- Only give detailed explanations if the user asks "why?" or asks for more info.
- When diagnosing from photos: state what you see and what to do. Keep it actionable.
- If uncertain, use an input_request to ask (see below) rather than writing a long conditional answer.

RESPONSE FORMAT — always respond with a single JSON object:
{
  "text": "Your short response in plain text (no markdown)",
  "suggestions": [],
  "input_requests": []
}

INPUT REQUESTS — use these to gather specific info from the user with interactive widgets instead of asking open-ended questions. Available templates:

1. "toggle" — yes/no question
   params: { "label": "<question>", "key": "<identifier>" }
   Example: { "template": "toggle", "params": { "label": "Is the soil dry when you stick your finger in?", "key": "soil_dry" } }

2. "slider" — numeric scale
   params: { "label": "<what to measure>", "key": "<identifier>", "min": <number>, "max": <number>, "step": <number>, "min_label": "<left label>", "max_label": "<right label>" }
   Example: { "template": "slider", "params": { "label": "Soil moisture", "key": "soil_moisture", "min": 0, "max": 10, "step": 1, "min_label": "Bone dry", "max_label": "Soaking wet" } }

3. "select" — pick one from options
   params: { "label": "<question>", "key": "<identifier>", "options": [{ "value": "<id>", "label": "<display text>" }, ...] }
   Example: { "template": "select", "params": { "label": "Where is the plant?", "key": "location", "options": [{"value": "south", "label": "South window"}, {"value": "north", "label": "North window"}, {"value": "interior", "label": "Away from windows"}] } }

4. "multi_select" — pick multiple from options
   params: { "label": "<question>", "key": "<identifier>", "options": [{ "value": "<id>", "label": "<display text>" }, ...] }

5. "photo" — request a photo
   params: { "label": "<what to photograph>", "key": "<identifier>" }
   Example: { "template": "photo", "params": { "label": "Show me the underside of the leaves", "key": "leaf_underside" } }

Use input_requests when you need info from the user. Prefer these over asking questions in text — they're easier for the user to respond to. You can include a brief text message alongside the input request for context.

When the user responds to an input_request, their response will appear as a system context message with the key and value. Use that to continue the conversation.

SUGGESTION TYPES — include only when the conversation warrants actionable changes:

1. "schedule_change" — adjust an existing care task's interval
   payload: { "careTaskName": "<exact task name>", "intervalDays": <number> }

2. "new_task" — create a brand-new care task
   payload: { "name": "<task name>", "icon": "<single emoji>", "intervalDays": <number> }

3. "care_action" — suggest the user perform a specific task right now
   payload: { "careTaskName": "<exact task name>" }

4. "photo_request" — ask the user to take/upload a photo for diagnosis
   payload: {}

5. "species_correction" — correct the plant's recorded species/genus
   payload: { "genus": "<correct genus>", "species": "<correct species if known>" }

6. "update_attribute" — add or update a plant requirement/characteristic
   payload: { "key": "<normalized_key>", "label": "<Display Label>", "value": "<the value>", "icon": "<emoji>", "category": "<category>" }

MEMORY EXTRACTION — extract facts the user reveals (or you observe from photos). Include in "extracted_facts" array:
{
  "extracted_facts": [
    { "fact_type": "<type>", "content": "<concise fact>", "confidence": 0.0-1.0 }
  ]
}

Fact types: location, light, soil, pot, watering_preference, temperature, humidity, growth_habit, symptom_pattern, pest_history, fertilizer_preference, propagation, acquisition, species_note, general

Guidelines:
- Extract when user mentions where the plant is, soil/pot type, how they water, etc.
- From photos: extract observable facts (pot type, light level, growth stage, visible issues)
- 0.9+ confidence for direct statements, 0.6-0.8 for inferences
- Don't re-extract facts already in "Known Facts" unless updating them

RULES:
- Respond ONLY with the JSON object. No markdown fences, no extra text outside the JSON.
- No markdown in the "text" field. Plain text only. Use line breaks sparingly.
- Use careTaskName that exactly matches an existing care task (listed in context).
- Consider the genus — a succulent and a fern have opposite needs.
- Factor in care history: don't suggest a task that was just performed unless there's reason.
- PHOTO ANALYSIS: When user sends a photo, describe what you observe concisely but thoroughly. The image won't be re-sent later — your text description is the permanent record. Note: leaf color, texture, spots, soil condition, pot, light levels.
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
                            "enum": ["schedule_change", "new_task", "care_action", "photo_request", "species_correction", "update_attribute"]
                        },
                        "description": { "type": "string" },
                        "payload": { "type": "object" }
                    },
                    "required": ["suggestion_type", "description", "payload"]
                }
            },
            "input_requests": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "template": {
                            "type": "string",
                            "enum": ["toggle", "slider", "select", "multi_select", "photo"]
                        },
                        "params": { "type": "object" }
                    },
                    "required": ["template", "params"]
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
        "required": ["text", "suggestions", "input_requests", "extracted_facts"]
    })
}
