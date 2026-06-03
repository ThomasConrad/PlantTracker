use anyhow::{Context, Result};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tracing::error;

use super::{ChatMessage, CoachResponse, ContentPart, PlantCoach, RESPONSE_JSON_SCHEMA};

pub struct OpenAICoach {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAICoach {
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .context("OPENAI_API_KEY environment variable not set")?;
        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
        let base_url = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .context("Failed to build reqwest client")?;

        Ok(Self {
            client,
            api_key,
            model,
            base_url,
        })
    }

    pub fn new_with_config(base_url: String, api_key: String, model: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .context("Failed to build reqwest client")?;

        Ok(Self {
            client,
            api_key,
            model,
            base_url,
        })
    }

    fn build_url(&self) -> String {
        format!(
            "{}/chat/completions",
            self.base_url.trim_end_matches('/')
        )
    }
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
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

#[derive(Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Deserialize)]
struct StreamDelta {
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

#[async_trait::async_trait]
impl PlantCoach for OpenAICoach {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<CoachResponse> {
        let schema: serde_json::Value =
            serde_json::from_str(RESPONSE_JSON_SCHEMA).context("Failed to parse JSON schema")?;

        let response_format = json!({
            "type": "json_schema",
            "json_schema": {
                "name": "coach_response",
                "strict": true,
                "schema": schema
            }
        });

        let openai_messages = convert_messages(messages);

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: openai_messages,
            response_format: Some(response_format),
            stream: None,
        };

        let response = self
            .client
            .post(self.build_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .context("Failed to send request to LLM provider")?;

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

    async fn stream_chat(
        &self,
        messages: Vec<ChatMessage>,
        tx: tokio::sync::mpsc::Sender<String>,
    ) -> Result<CoachResponse> {
        let schema: serde_json::Value =
            serde_json::from_str(RESPONSE_JSON_SCHEMA).context("Failed to parse JSON schema")?;

        let response_format = json!({
            "type": "json_schema",
            "json_schema": {
                "name": "coach_response",
                "strict": true,
                "schema": schema
            }
        });

        let openai_messages = convert_messages(messages);

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: openai_messages,
            response_format: Some(response_format),
            stream: Some(true),
        };

        let response = self
            .client
            .post(self.build_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .context("Failed to send request to LLM provider")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("OpenAI streaming error: status={}, body={}", status, body);
            anyhow::bail!("OpenAI API returned status {}: {}", status, body);
        }

        let mut full_content = String::new();
        let mut stream = response.bytes_stream();

        // Track if we're inside the "text" field value to forward tokens
        let mut in_text_field = false;
        let mut text_buffer = String::new();
        let mut brace_depth = 0;
        let mut sent_chars = 0;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Stream chunk error")?;
            let chunk_str = String::from_utf8_lossy(&chunk);

            for line in chunk_str.lines() {
                let line = line.trim();
                if line.is_empty() || line == "data: [DONE]" {
                    continue;
                }
                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(parsed) = serde_json::from_str::<StreamChunk>(data) {
                        for choice in parsed.choices {
                            if let Some(content) = choice.delta.content {
                                full_content.push_str(&content);

                                // Heuristic: try to extract text field content progressively
                                // Once we see `"text":"` or `"text": "`, start forwarding
                                if !in_text_field {
                                    if let Some(pos) = full_content.find("\"text\"") {
                                        // Find the opening quote of the value
                                        let after = &full_content[pos + 6..];
                                        if let Some(quote_pos) = after.find('"') {
                                            in_text_field = true;
                                            let value_start = pos + 6 + quote_pos + 1;
                                            text_buffer =
                                                full_content[value_start..].to_string();
                                            // Track brace depth for nested JSON
                                            brace_depth = 0;
                                        }
                                    }
                                } else {
                                    text_buffer.push_str(&content);
                                }

                                // Forward new text to the client
                                if in_text_field && text_buffer.len() > sent_chars {
                                    // Check if we've hit the end of the text field
                                    // Look for unescaped closing quote
                                    let _check = &text_buffer[sent_chars..];
                                    // Don't send the last few chars in case they're
                                    // part of an escape sequence or the closing quote
                                    let safe_end =
                                        find_safe_send_boundary(&text_buffer, sent_chars);
                                    if safe_end > sent_chars {
                                        let to_send = &text_buffer[sent_chars..safe_end];
                                        // Unescape JSON string escapes
                                        let unescaped = unescape_json_string(to_send);
                                        if !unescaped.is_empty() {
                                            let _ = tx.send(unescaped).await;
                                        }
                                        sent_chars = safe_end;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Send any remaining text
        if in_text_field && text_buffer.len() > sent_chars {
            // Remove trailing quote if present
            let remaining = &text_buffer[sent_chars..];
            let remaining = remaining.trim_end_matches('"');
            if !remaining.is_empty() {
                let unescaped = unescape_json_string(remaining);
                if !unescaped.is_empty() {
                    let _ = tx.send(unescaped).await;
                }
            }
        }

        // Parse the complete response
        let _ = brace_depth; // suppress unused warning
        let coach_response: CoachResponse = serde_json::from_str(&full_content)
            .context("Failed to parse streamed coach response JSON")?;

        Ok(coach_response)
    }
}

/// Find a safe boundary to send text up to (avoid splitting escape sequences)
fn find_safe_send_boundary(buffer: &str, from: usize) -> usize {
    let bytes = buffer.as_bytes();
    let len = bytes.len();
    // Don't send the last 2 chars (might be `\"` or end quote)
    if len <= from + 2 {
        return from;
    }
    let mut end = len - 2;
    // Make sure we don't split in the middle of a backslash escape
    while end > from && bytes[end - 1] == b'\\' {
        end -= 1;
    }
    end
}

/// Basic JSON string unescaping
fn unescape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('/') => result.push('/'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}
