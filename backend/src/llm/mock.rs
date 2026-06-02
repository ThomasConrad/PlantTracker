//! Mock LLM provider for testing.
//! Returns deterministic responses based on the user's message content.
//! Recognizes keywords to generate specific suggestion types.

use anyhow::Result;

use super::{ChatMessage, CoachResponse, CoachSuggestionOutput, ContentPart, PlantCoach};

/// A mock coach that returns predictable suggestions based on message keywords.
///
/// Keywords in the last user message trigger specific suggestions:
/// - "schedule" → schedule_change suggestion (Water → 5 days)
/// - "new task" → new_task suggestion (Misting)
/// - "action" → care_action suggestion (Water)
/// - "photo" → photo_request suggestion
/// - anything else → text-only response with no suggestions
pub struct MockCoach;

#[async_trait::async_trait]
impl PlantCoach for MockCoach {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<CoachResponse> {
        // Find the last user message text
        let last_user_text = messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .and_then(|m| {
                m.content.iter().find_map(|part| match part {
                    ContentPart::Text { text } => Some(text.as_str()),
                    _ => None,
                })
            })
            .unwrap_or("");

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

        let text = if suggestions.is_empty() {
            "Your plant looks healthy! Keep up the good work.".to_string()
        } else {
            format!(
                "I've analyzed your plant and have {} suggestion(s) for you.",
                suggestions.len()
            )
        };

        Ok(CoachResponse { text, suggestions })
    }
}
