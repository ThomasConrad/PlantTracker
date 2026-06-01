pub mod anthropic;
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
    pub text: String,
    pub suggestions: Vec<CoachSuggestionOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachSuggestionOutput {
    pub suggestion_type: String, // "schedule_change" | "reminder" | "care_action" | "photo_request"
    pub description: String,
    pub payload: serde_json::Value,
}

#[async_trait::async_trait]
pub trait PlantCoach: Send + Sync {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<CoachResponse>;
}

/// Factory: create the coach from environment config
pub fn create_coach() -> Result<Box<dyn PlantCoach>> {
    let provider =
        std::env::var("PLANT_COACH_PROVIDER").unwrap_or_else(|_| "openai".to_string());
    match provider.as_str() {
        "anthropic" => Ok(Box::new(anthropic::AnthropicCoach::new()?)),
        _ => Ok(Box::new(openai::OpenAICoach::new()?)),
    }
}
