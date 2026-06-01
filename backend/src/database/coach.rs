use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use crate::database::DatabasePool;
use crate::models::coach::*;

/// Get or create a conversation for a plant+user pair
pub async fn get_or_create_conversation(
    pool: &DatabasePool,
    plant_id: &str,
    user_id: &str,
) -> Result<CoachConversationRow> {
    // Try to find existing conversation
    let existing: Option<CoachConversationRow> = sqlx::query_as(
        "SELECT id, plant_id, user_id, title, created_at, updated_at
         FROM coach_conversations
         WHERE plant_id = ? AND user_id = ?",
    )
    .bind(plant_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if let Some(conversation) = existing {
        return Ok(conversation);
    }

    // Create new conversation
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let row: CoachConversationRow = sqlx::query_as(
        "INSERT INTO coach_conversations (id, plant_id, user_id, title, created_at, updated_at)
         VALUES (?, ?, ?, NULL, ?, ?)
         RETURNING id, plant_id, user_id, title, created_at, updated_at",
    )
    .bind(&id)
    .bind(plant_id)
    .bind(user_id)
    .bind(&now)
    .bind(&now)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Get all messages for a conversation, ordered by created_at
pub async fn get_messages(
    pool: &DatabasePool,
    conversation_id: &str,
) -> Result<Vec<CoachMessageRow>> {
    let messages: Vec<CoachMessageRow> = sqlx::query_as(
        "SELECT id, conversation_id, role, content, image_url, created_at
         FROM coach_messages
         WHERE conversation_id = ?
         ORDER BY created_at ASC",
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await?;

    Ok(messages)
}

/// Get suggestions for a specific message
pub async fn get_suggestions_for_message(
    pool: &DatabasePool,
    message_id: &str,
) -> Result<Vec<CoachSuggestionRow>> {
    let suggestions: Vec<CoachSuggestionRow> = sqlx::query_as(
        "SELECT id, message_id, plant_id, suggestion_type, payload, status, applied_at, created_at
         FROM coach_suggestions
         WHERE message_id = ?",
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;

    Ok(suggestions)
}

/// Get all pending suggestions for a plant
pub async fn get_pending_suggestions(
    pool: &DatabasePool,
    plant_id: &str,
    user_id: &str,
) -> Result<Vec<CoachSuggestionRow>> {
    let suggestions: Vec<CoachSuggestionRow> = sqlx::query_as(
        "SELECT s.id, s.message_id, s.plant_id, s.suggestion_type, s.payload, s.status, s.applied_at, s.created_at
         FROM coach_suggestions s
         JOIN coach_messages m ON m.id = s.message_id
         JOIN coach_conversations c ON c.id = m.conversation_id
         WHERE s.plant_id = ? AND c.user_id = ? AND s.status = 'pending'",
    )
    .bind(plant_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(suggestions)
}

/// Insert a new message
pub async fn insert_message(
    pool: &DatabasePool,
    conversation_id: &str,
    role: &str,
    content: &str,
    image_url: Option<&str>,
) -> Result<CoachMessageRow> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let row: CoachMessageRow = sqlx::query_as(
        "INSERT INTO coach_messages (id, conversation_id, role, content, image_url, created_at)
         VALUES (?, ?, ?, ?, ?, ?)
         RETURNING id, conversation_id, role, content, image_url, created_at",
    )
    .bind(&id)
    .bind(conversation_id)
    .bind(role)
    .bind(content)
    .bind(image_url)
    .bind(&now)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Insert a suggestion linked to a message
pub async fn insert_suggestion(
    pool: &DatabasePool,
    message_id: &str,
    plant_id: &str,
    suggestion_type: &str,
    payload: &serde_json::Value,
) -> Result<CoachSuggestionRow> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let payload_str = serde_json::to_string(payload)?;

    let row: CoachSuggestionRow = sqlx::query_as(
        "INSERT INTO coach_suggestions (id, message_id, plant_id, suggestion_type, payload, status, applied_at, created_at)
         VALUES (?, ?, ?, ?, ?, 'pending', NULL, ?)
         RETURNING id, message_id, plant_id, suggestion_type, payload, status, applied_at, created_at",
    )
    .bind(&id)
    .bind(message_id)
    .bind(plant_id)
    .bind(suggestion_type)
    .bind(&payload_str)
    .bind(&now)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Update suggestion status (accept or dismiss)
pub async fn update_suggestion_status(
    pool: &DatabasePool,
    suggestion_id: &str,
    user_id: &str,
    status: &str,
) -> Result<Option<CoachSuggestionRow>> {
    let applied_at = if status == "accepted" {
        Some(Utc::now().to_rfc3339())
    } else {
        None
    };

    let row: Option<CoachSuggestionRow> = sqlx::query_as(
        "UPDATE coach_suggestions
         SET status = ?, applied_at = ?
         WHERE id = ? AND plant_id IN (
             SELECT plant_id FROM coach_conversations WHERE user_id = ?
         )
         RETURNING id, message_id, plant_id, suggestion_type, payload, status, applied_at, created_at",
    )
    .bind(status)
    .bind(&applied_at)
    .bind(suggestion_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}
