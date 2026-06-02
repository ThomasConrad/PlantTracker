use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::error;

use super::{ChatMessage, CoachResponse, ContentPart, PlantCoach, RESPONSE_JSON_SCHEMA};

/// Ollama coach — uses Ollama's native `/api/chat` endpoint.
/// Configure via:
///   OLLAMA_URL (default: http://localhost:11434)
///   OLLAMA_MODEL (default: llama3.2-vision)
pub struct OllamaCoach {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaCoach {
    pub fn new() -> Result<Self> {
        let base_url = std::env::var("OLLAMA_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        let model = std::env::var("OLLAMA_MODEL")
            .unwrap_or_else(|_| "llama3.2-vision".to_string());
        let client = Client::builder()
            .timeout(Duration::from_secs(120)) // local models can be slower
            .build()
            .context("Failed to build reqwest client")?;

        Ok(Self {
            client,
            base_url,
            model,
        })
    }
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    format: serde_json::Value,
    stream: bool,
}

#[derive(Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    images: Vec<String>,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: Option<OllamaResponseMessage>,
}

#[derive(Deserialize)]
struct OllamaResponseMessage {
    content: Option<String>,
}

/// Convert our generic messages into Ollama's format.
/// Ollama uses a flat `content` string + separate `images` array (raw base64).
fn convert_messages(messages: Vec<ChatMessage>) -> Vec<OllamaMessage> {
    messages
        .into_iter()
        .map(|msg| {
            let mut text_parts = Vec::new();
            let mut images = Vec::new();

            for part in msg.content {
                match part {
                    ContentPart::Text { text } => text_parts.push(text),
                    ContentPart::ImageUrl { image_url } => {
                        // Ollama expects raw base64 without the "data:image/...;base64," prefix
                        let url = image_url.url;
                        if let Some(base64_data) = url.split(',').nth(1) {
                            images.push(base64_data.to_string());
                        } else {
                            images.push(url);
                        }
                    }
                }
            }

            OllamaMessage {
                role: msg.role,
                content: text_parts.join("\n"),
                images,
            }
        })
        .collect()
}

#[async_trait::async_trait]
impl PlantCoach for OllamaCoach {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<CoachResponse> {
        let format: serde_json::Value =
            serde_json::from_str(RESPONSE_JSON_SCHEMA).context("Failed to parse response schema")?;

        let ollama_messages = convert_messages(messages);

        let request = OllamaRequest {
            model: self.model.clone(),
            messages: ollama_messages,
            format,
            stream: false,
        };

        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Ollama API error: status={}, body={}", status, body);
            anyhow::bail!("Ollama API returned status {}: {}", status, body);
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        let content = ollama_response
            .message
            .and_then(|m| m.content)
            .context("No content in Ollama response")?;

        let coach_response: CoachResponse =
            serde_json::from_str(&content).context("Failed to parse coach response JSON from Ollama")?;

        Ok(coach_response)
    }
}
