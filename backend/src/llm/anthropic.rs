use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::error;

use super::{ChatMessage, CoachResponse, ContentPart, PlantCoach};

pub struct AnthropicCoach {
    client: Client,
    api_key: String,
    model: String,
}

impl AnthropicCoach {
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .context("ANTHROPIC_API_KEY environment variable not set")?;
        let model = std::env::var("ANTHROPIC_MODEL")
            .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string());
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
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<AnthropicMessage>,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<AnthropicContent>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum AnthropicContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: AnthropicImageSource },
}

#[derive(Serialize)]
struct AnthropicImageSource {
    #[serde(rename = "type")]
    source_type: String,
    media_type: String,
    data: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicResponseContent>,
}

#[derive(Deserialize)]
struct AnthropicResponseContent {
    text: Option<String>,
}

fn convert_messages(messages: Vec<ChatMessage>) -> (String, Vec<AnthropicMessage>) {
    let mut system = String::new();
    let mut anthropic_messages = Vec::new();

    for msg in messages {
        if msg.role == "system" {
            // Append any system messages to the system prompt
            for part in &msg.content {
                if let ContentPart::Text { text } = part {
                    if !system.is_empty() {
                        system.push('\n');
                    }
                    system.push_str(text);
                }
            }
            continue;
        }

        let mut content = Vec::new();
        for part in msg.content {
            match part {
                ContentPart::Text { text } => {
                    content.push(AnthropicContent::Text { text });
                }
                ContentPart::ImageUrl { image_url } => {
                    // Parse data URL: data:image/jpeg;base64,<data>
                    if let Some((header, data)) = image_url.url.split_once(',') {
                        let media_type = header
                            .strip_prefix("data:")
                            .and_then(|s| s.split(';').next())
                            .unwrap_or("image/jpeg")
                            .to_string();
                        content.push(AnthropicContent::Image {
                            source: AnthropicImageSource {
                                source_type: "base64".to_string(),
                                media_type,
                                data: data.to_string(),
                            },
                        });
                    }
                }
            }
        }

        anthropic_messages.push(AnthropicMessage {
            role: msg.role,
            content,
        });
    }

    (system, anthropic_messages)
}

#[async_trait::async_trait]
impl PlantCoach for AnthropicCoach {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<CoachResponse> {
        let (system, anthropic_messages) = convert_messages(messages);

        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 2048,
            system,
            messages: anthropic_messages,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Anthropic")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Anthropic API error: status={}, body={}", status, body);
            anyhow::bail!("Anthropic API returned status {}: {}", status, body);
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .context("Failed to parse Anthropic response")?;

        let content = anthropic_response
            .content
            .into_iter()
            .find_map(|c| c.text)
            .context("No text content in Anthropic response")?;

        // Strip markdown fences if present (Anthropic sometimes wraps despite instructions)
        let clean = content
            .trim()
            .strip_prefix("```json")
            .or_else(|| content.trim().strip_prefix("```"))
            .unwrap_or(&content)
            .trim_end_matches("```")
            .trim();

        let coach_response: CoachResponse = serde_json::from_str(clean)
            .context("Failed to parse coach response JSON from Anthropic")?;

        Ok(coach_response)
    }
}
