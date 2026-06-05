use anyhow::{Context, Result};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tracing::error;

use super::{ChatMessage, CoachResponse, ContentPart, PlantCoach, coach_response_schema};

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
        let schema = coach_response_schema();

        let response_format = json!({
            "type": "json_schema",
            "json_schema": {
                "name": "coach_response",
                "strict": false,
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

        // Strip markdown code fences if model wrapped response
        let content_trimmed = content.trim();
        let content_trimmed = content_trimmed
            .strip_prefix("```json")
            .or_else(|| content_trimmed.strip_prefix("```"))
            .unwrap_or(content_trimmed);
        let content_trimmed = content_trimmed
            .strip_suffix("```")
            .unwrap_or(content_trimmed)
            .trim();

        let coach_response: CoachResponse =
            serde_json::from_str(content_trimmed).map_err(|e| {
                error!(
                    "Failed to parse coach response. Error: {}. Raw content ({} bytes): {}",
                    e,
                    content.len(),
                    &content[..content.len().min(500)]
                );
                anyhow::anyhow!("Failed to parse coach response JSON: {e}")
            })?;

        Ok(coach_response)
    }

    async fn chat_raw(
        &self,
        messages: Vec<ChatMessage>,
        response_schema: Option<serde_json::Value>,
    ) -> Result<String> {
        let response_format = response_schema.map(|schema| {
            json!({
                "type": "json_schema",
                "json_schema": {
                    "name": "structured_response",
                    "strict": false,
                    "schema": schema
                }
            })
        });

        let openai_messages = convert_messages(messages);

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: openai_messages,
            response_format,
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

        // Strip markdown code fences if model wrapped response
        let content = content.trim();
        let content = content
            .strip_prefix("```json")
            .or_else(|| content.strip_prefix("```"))
            .unwrap_or(content);
        let content = content.strip_suffix("```").unwrap_or(content).trim();

        Ok(content.to_string())
    }

    async fn stream_chat(
        &self,
        messages: Vec<ChatMessage>,
        tx: tokio::sync::mpsc::Sender<String>,
    ) -> Result<CoachResponse> {
        let schema = coach_response_schema();

        let response_format = json!({
            "type": "json_schema",
            "json_schema": {
                "name": "coach_response",
                "strict": false,
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

        // State machine for extracting the "text" field value from streaming JSON
        let mut extractor = TextFieldExtractor::new();

        // Buffer for incomplete SSE lines split across TCP chunks
        let mut line_buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Stream chunk error")?;
            let chunk_str = String::from_utf8_lossy(&chunk);

            line_buffer.push_str(&chunk_str);

            // Process all complete lines (terminated by \n)
            while let Some(newline_pos) = line_buffer.find('\n') {
                let line = line_buffer[..newline_pos].trim().to_string();
                line_buffer = line_buffer[newline_pos + 1..].to_string();

                if line.is_empty() || line == "data: [DONE]" {
                    continue;
                }
                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(parsed) = serde_json::from_str::<StreamChunk>(data) {
                        for choice in parsed.choices {
                            if let Some(content) = choice.delta.content {
                                full_content.push_str(&content);

                                // Feed chars to the extractor; it emits decoded text
                                // only while inside the "text" field value.
                                let extracted = extractor.feed(&content);
                                if !extracted.is_empty() {
                                    let _ = tx.send(extracted).await;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Parse the complete response
        let content = full_content.trim();
        // Strip markdown code fences if model wrapped response
        let content = content
            .strip_prefix("```json")
            .or_else(|| content.strip_prefix("```"))
            .unwrap_or(content);
        let content = content.strip_suffix("```").unwrap_or(content).trim();

        let coach_response: CoachResponse = serde_json::from_str(content)
            .map_err(|e| {
                error!(
                    "Failed to parse streamed coach response. Error: {}. Raw content ({} bytes): {}",
                    e,
                    content.len(),
                    &content[..content.len().min(500)]
                );
                anyhow::anyhow!("Failed to parse streamed coach response JSON: {e}")
            })?;

        Ok(coach_response)
    }
}

/// State machine that extracts the decoded text content of the `"text"` field
/// from a stream of JSON characters.  It handles:
/// - Finding `"text"` key followed by `:` and opening `"`
/// - Proper escape sequence tracking (`\"`, `\\`, `\n`, etc.)
/// - Detecting the unescaped closing `"` to stop extraction
#[derive(Debug)]
enum ExtractorState {
    /// Scanning for the key `"text"` in the JSON stream
    LookingForKey { ring: [u8; 6], ring_len: usize },
    /// Found the key, now looking for `:` then opening `"`
    WaitingForColon,
    WaitingForOpenQuote,
    /// Inside the string value, emitting decoded characters
    InsideValue { escaped: bool },
    /// Done — text field fully extracted, ignore rest
    Done,
}

struct TextFieldExtractor {
    state: ExtractorState,
}

impl TextFieldExtractor {
    fn new() -> Self {
        Self {
            state: ExtractorState::LookingForKey {
                ring: [0; 6],
                ring_len: 0,
            },
        }
    }

    /// Feed a chunk of characters. Returns decoded text to send to the client.
    fn feed(&mut self, input: &str) -> String {
        let mut output = String::new();

        for ch in input.chars() {
            match &mut self.state {
                ExtractorState::LookingForKey { ring, ring_len } => {
                    // We're looking for the 6 bytes: "text" (with quotes)
                    let b = ch as u8;
                    if *ring_len < 6 {
                        ring[*ring_len] = b;
                        *ring_len += 1;
                    } else {
                        // Shift left
                        ring.copy_within(1..6, 0);
                        ring[5] = b;
                    }
                    if *ring_len == 6 && ring == b"\"text\"" {
                        self.state = ExtractorState::WaitingForColon;
                    }
                }
                ExtractorState::WaitingForColon => {
                    if ch == ':' {
                        self.state = ExtractorState::WaitingForOpenQuote;
                    } else if !ch.is_whitespace() {
                        // Unexpected char — probably a different key that ends with "text"
                        // Go back to looking
                        self.state = ExtractorState::LookingForKey {
                            ring: [0; 6],
                            ring_len: 0,
                        };
                    }
                }
                ExtractorState::WaitingForOpenQuote => {
                    if ch == '"' {
                        self.state = ExtractorState::InsideValue { escaped: false };
                    } else if !ch.is_whitespace() {
                        // Value isn't a string — not the field we want
                        self.state = ExtractorState::LookingForKey {
                            ring: [0; 6],
                            ring_len: 0,
                        };
                    }
                }
                ExtractorState::InsideValue { escaped } => {
                    if *escaped {
                        // Previous char was backslash — decode escape
                        match ch {
                            'n' => output.push('\n'),
                            't' => output.push('\t'),
                            'r' => output.push('\r'),
                            '"' => output.push('"'),
                            '\\' => output.push('\\'),
                            '/' => output.push('/'),
                            // Unicode escapes (\uXXXX) — push raw for now
                            _ => {
                                output.push('\\');
                                output.push(ch);
                            }
                        }
                        *escaped = false;
                    } else if ch == '\\' {
                        *escaped = true;
                    } else if ch == '"' {
                        // Unescaped quote — end of text field value
                        self.state = ExtractorState::Done;
                    } else {
                        output.push(ch);
                    }
                }
                ExtractorState::Done => {
                    // Ignore everything after text field is complete
                }
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extractor_handles_typical_stream_chunks() {
        let mut ext = TextFieldExtractor::new();
        // Simulates actual OpenAI stream deltas: {"text":"I can help you!","suggestions":...}
        let chunks = vec![
            "{\"",
            "text",
            "\":\"",
            "I",
            " can",
            " help",
            " you",
            "!\",\"suggestions\":[],\"extracted_facts\":[]}",
        ];

        let mut result = String::new();
        for chunk in &chunks {
            result.push_str(&ext.feed(chunk));
        }
        assert_eq!(result, "I can help you!");
    }

    #[test]
    fn extractor_handles_escape_sequences() {
        let mut ext = TextFieldExtractor::new();
        // JSON: {"text":"line1\nline2\"quoted\"","suggestions":[]}
        let input = "{\"text\":\"line1\\nline2\\\"quoted\\\"\",\"suggestions\":[]}";

        let result = ext.feed(input);
        assert_eq!(result, "line1\nline2\"quoted\"");
    }

    #[test]
    fn extractor_ignores_content_after_text_field() {
        let mut ext = TextFieldExtractor::new();
        let input = "{\"text\":\"hello\",\"suggestions\":[{\"suggestion_type\":\"photo_request\",\"description\":\"Send a photo\",\"payload\":{}}],\"extracted_facts\":[]}";

        let result = ext.feed(input);
        assert_eq!(result, "hello");
    }

    #[test]
    fn extractor_handles_whitespace_around_colon() {
        let mut ext = TextFieldExtractor::new();
        let input = "{\"text\" : \"spaced out\"}";

        let result = ext.feed(input);
        assert_eq!(result, "spaced out");
    }

    #[test]
    fn extractor_handles_single_char_deltas() {
        let mut ext = TextFieldExtractor::new();
        let input = "{\"text\":\"Hi there!\"}";

        let mut result = String::new();
        for ch in input.chars() {
            result.push_str(&ext.feed(&ch.to_string()));
        }
        assert_eq!(result, "Hi there!");
    }
}

