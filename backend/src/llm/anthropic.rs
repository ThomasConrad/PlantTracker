use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tracing::error;

use super::{ChatMessage, CoachResponse, ContentPart, ImageUrlContent, PlantCoach};

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
    source_type: String, // "base64" or "url"
    media_type: String,
    data: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicResponseContent>,
}

#[derive(Deserialize)]
struct AnthropicResponseContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

const SYSTEM_PROMPT: &str = r#"You are a helpful plant care coach. Always respond with valid JSON matching this exact schema:
{
  "text": "Your natural language response to the user",
  "suggestions": [
    {
      "suggestion_type": "schedule_change|reminder|care_action|photo_request",
      "description": "Human-readable description of the suggestion",
      "payload": {}
    }
  ]
}
Respond ONLY with the JSON object, no additional text or markdown formatting."#;

fn convert_messages(messages: Vec<ChatMessage>) -> (String, Vec<AnthropicMessage>) {
    let mut system = SYSTEM_PROMPT.to_string();
    let mut anthropic_messages = Vec::new();

    for msg in messages {
        if msg.role == "system" {
            // Append any user-provided system messages to our system prompt
            for part in &msg.content {
                if let ContentPart::Text { text } = part {
                    system.push_str("\n\n");
                    system.push_str(text);
                }
            }
            continue;
        }

        let content: Vec<AnthropicContent> = msg
            .content
            .into_iter()
            .map(|part| match part {
                ContentPart::Text { text } => AnthropicContent::Text { text },
                ContentPart::ImageUrl { image_url } => {
                    // Parse base64 data URLs: data:image/png;base64,<data>
                    if let Some(rest) = image_url.url.strip_prefix("data:") {
                        if let Some((meta, data)) = rest.split_once(",") {
                            let media_type = meta.replace(";base64", "");
                            return AnthropicContent::Image {
                                source: AnthropicImageSource {
                                    source_type: "base64".to_string(),
                                    media_type,
                                    data: data.to_string(),
                                },
                            };
                        }
                    }
                    // Fallback: treat as URL reference in text
                    AnthropicContent::Text {
                        text: format!("[Image: {}]", image_url.url),
                    }
                }
            })
            .collect();

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
            max_tokens: 4096,
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

        let text_content = anthropic_response
            .content
            .into_iter()
            .find(|c| c.content_type == "text")
            .and_then(|c| c.text)
            .context("No text content in Anthropic response")?;

        let coach_response: CoachResponse = serde_json::from_str(&text_content)
            .context("Failed to parse coach response JSON from Anthropic")?;

        Ok(coach_response)
    }
}
