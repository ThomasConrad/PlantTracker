//! Mock LLM provider for testing.
//! Returns deterministic responses based on the user's message content.
//! Recognizes keywords to generate specific suggestion types.

use anyhow::Result;

use super::{
    ChatMessage, CoachResponse, CoachSuggestionOutput, ContentPart, ExtractedFactOutput, PlantCoach,
};

/// A mock coach that returns predictable suggestions based on message keywords.
///
/// Keywords in the last user message trigger specific suggestions:
/// - "schedule" → schedule_change suggestion (Water → 5 days)
/// - "new task" → new_task suggestion (Misting)
/// - "action" → care_action suggestion (Water)
/// - "photo" → photo_request suggestion
/// - anything else → text-only response with no suggestions
///
/// If the last user message includes an image, the response acknowledges it
/// with a photo analysis preamble.
pub struct MockCoach;

#[async_trait::async_trait]
impl PlantCoach for MockCoach {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<CoachResponse> {
        // Find the last user message
        let last_user = messages.iter().rev().find(|m| m.role == "user");

        let last_user_text = last_user
            .and_then(|m| {
                m.content.iter().find_map(|part| match part {
                    ContentPart::Text { text } => Some(text.as_str()),
                    _ => None,
                })
            })
            .unwrap_or("");

        let has_image = last_user
            .map(|m| {
                m.content
                    .iter()
                    .any(|part| matches!(part, ContentPart::ImageUrl { .. }))
            })
            .unwrap_or(false);

        let lower = last_user_text.to_lowercase();

        let mut suggestions = Vec::new();

        if lower.contains("schedule") {
            suggestions.push(CoachSuggestionOutput {
                suggestion_type: "schedule_change".to_string(),
                description: "Water more frequently — every 5 days".to_string(),
                payload: serde_json::json!({
                    "careTaskName": "Water",
                    "intervalDays": 5
                }),
            });
        }

        if lower.contains("new task") {
            suggestions.push(CoachSuggestionOutput {
                suggestion_type: "new_task".to_string(),
                description: "Add a misting task for humidity".to_string(),
                payload: serde_json::json!({
                    "name": "Misting",
                    "icon": "💨",
                    "intervalDays": 2
                }),
            });
        }

        if lower.contains("action") {
            suggestions.push(CoachSuggestionOutput {
                suggestion_type: "care_action".to_string(),
                description: "Water your plant now — it looks thirsty".to_string(),
                payload: serde_json::json!({
                    "careTaskName": "Water"
                }),
            });
        }

        if lower.contains("photo") {
            suggestions.push(CoachSuggestionOutput {
                suggestion_type: "photo_request".to_string(),
                description: "Can you take a photo of the leaves?".to_string(),
                payload: serde_json::json!({}),
            });
        }

        let text = if has_image && suggestions.is_empty() {
            "I can see your plant in the photo! The leaves look healthy with good coloration. I notice some new growth forming — that's a great sign. Keep up your current care routine.".to_string()
        } else if has_image {
            format!(
                "I've analyzed the photo of your plant and have {} suggestion(s) based on what I see.",
                suggestions.len()
            )
        } else if suggestions.is_empty() {
            "Your plant looks healthy! Keep up the good work.".to_string()
        } else {
            format!(
                "I've analyzed your plant and have {} suggestion(s) for you.",
                suggestions.len()
            )
        };

        // Extract mock facts based on keywords
        let mut extracted_facts = Vec::new();
        if lower.contains("window") || lower.contains("south") || lower.contains("north") {
            extracted_facts.push(ExtractedFactOutput {
                fact_type: "location".to_string(),
                content: "Near a window".to_string(),
                confidence: 0.8,
            });
        }
        if lower.contains("terracotta") || lower.contains("pot") {
            extracted_facts.push(ExtractedFactOutput {
                fact_type: "pot".to_string(),
                content: "In a terracotta pot".to_string(),
                confidence: 0.85,
            });
        }
        if has_image {
            extracted_facts.push(ExtractedFactOutput {
                fact_type: "general".to_string(),
                content: "Plant appears healthy with good leaf coloration".to_string(),
                confidence: 0.7,
            });
        }

        Ok(CoachResponse {
            text,
            suggestions,
            extracted_facts,
        })
    }

    async fn stream_chat(
        &self,
        messages: Vec<ChatMessage>,
        tx: tokio::sync::mpsc::Sender<String>,
    ) -> Result<CoachResponse> {
        let response = self.chat(messages).await?;
        // Simulate word-by-word streaming with small delays
        for word in response.text.split_inclusive(' ') {
            let _ = tx.send(word.to_string()).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
        }
        Ok(response)
    }
}
