use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tracing::error;

use super::{ChatMessage, CoachResponse, ContentPart, PlantCoach};

pub struct OpenAICoach {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAICoach {
    pub fn new() -> Result<Self> {
        let api_key =
            std::env::var("OPENAI_API_KEY").context("OPENAI_API_KEY environment variable not set")?;
        let model =
            std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .context("Failed to build reqwest client")?;

        Ok(Self {
            client,
            api_key,
            model,
        })
    }
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    response_format: serde_json::Value,
}

#[derive(Serialize)]
struct OpenAIMessage {
    role: String,
    content: serde_json::Value,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
}

#[derive(Deserialize)]
struct OpenAIResponseMessage {
    content: Option<String>,
}

fn convert_messages(messages: Vec<ChatMessage>) -> Vec<OpenAIMessage> {
    messages
        .into_iter()
        .map(|msg| {
            let content = if msg.content.len() == 1 {
                if let ContentPart::Text { ref text } = msg.content[0] {
                    serde_json::Value::String(text.clone())
                } else {
                    serde_json::to_value(&msg.content).unwrap_or_default()
                }
            } else {
                serde_json::to_value(&msg.content).unwrap_or_default()
            };
            OpenAIMessage {
                role: msg.role,
                content,
            }
        })
        .collect()
}

const JSON_SCHEMA: &str = r#"{
    "type": "json_schema",
    "json_schema": {
        "name": "coach_response",
        "strict": true,
        "schema": {
            "type": "object",
            "properties": {
                "text": { "type": "string" },
                "suggestions": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "suggestion_type": { "type": "string" },
                            "description": { "type": "string" },
                            "payload": {}
                        },
                        "required": ["suggestion_type", "description", "payload"],
                        "additionalProperties": false
                    }
                }
            },
            "required": ["text", "suggestions"],
            "additionalProperties": false
        }
    }
}"#;

#[async_trait::async_trait]
impl PlantCoach for OpenAICoach {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<CoachResponse> {
        let response_format: serde_json::Value =
            serde_json::from_str(JSON_SCHEMA).context("Failed to parse JSON schema")?;

        let openai_messages = convert_messages(messages);

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: openai_messages,
            response_format,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .context("Failed to send request to OpenAI")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("OpenAI API error: status={}, body={}", status, body);
            anyhow::bail!("OpenAI API returned status {}: {}", status, body);
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .context("Failed to parse OpenAI response")?;

        let content = openai_response
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .context("No content in OpenAI response")?;

        let coach_response: CoachResponse =
            serde_json::from_str(&content).context("Failed to parse coach response JSON")?;

        Ok(coach_response)
    }
}
