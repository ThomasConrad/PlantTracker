use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::AuthSession;
use crate::database::coach as db_coach;
use crate::database::plants as db_plants;
use crate::llm::{ChatMessage, ContentPart};
use crate::middleware::validation::ValidatedJson;
use crate::models::coach::*;
use crate::utils::errors::{AppError, Result};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/plants/:plant_id/messages", get(get_messages).post(send_message))
        .route("/suggestions/:suggestion_id/accept", post(accept_suggestion))
        .route("/suggestions/:suggestion_id/dismiss", post(dismiss_suggestion))
}

#[utoipa::path(
    get,
    path = "/coach/plants/{plant_id}/messages",
    responses(
        (status = 200, description = "Conversation messages", body = CoachMessagesResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Plant not found"),
    ),
    params(
        ("plant_id" = Uuid, Path, description = "Plant ID")
    ),
    security(
        ("session" = [])
    ),
    tag = "coach"
)]
async fn get_messages(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path(plant_id): Path<Uuid>,
) -> Result<Json<CoachMessagesResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    // Verify plant belongs to user
    let plant = db_plants::get_plant_by_id(&app_state.pool, plant_id).await?;
    if plant.user_id != user.id {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    let conversation =
        db_coach::get_or_create_conversation(&app_state.pool, &plant_id.to_string(), &user.id)
            .await
            .map_err(|e| AppError::Internal {
                message: e.to_string(),
            })?;

    let message_rows = db_coach::get_messages(&app_state.pool, &conversation.id)
        .await
        .map_err(|e| AppError::Internal {
            message: e.to_string(),
        })?;

    let mut messages = Vec::new();
    for row in message_rows {
        let suggestion_rows = db_coach::get_suggestions_for_message(&app_state.pool, &row.id)
            .await
            .map_err(|e| AppError::Internal {
                message: e.to_string(),
            })?;

        let suggestions = suggestion_rows
            .into_iter()
            .map(|s| CoachSuggestion {
                id: s.id,
                suggestion_type: s.suggestion_type,
                payload: serde_json::from_str(&s.payload).unwrap_or_default(),
                status: s.status,
                applied_at: s.applied_at,
            })
            .collect();

        messages.push(CoachMessage {
            id: row.id,
            role: row.role,
            content: row.content,
            image_url: row.image_url,
            suggestions,
            created_at: row.created_at,
        });
    }

    Ok(Json(CoachMessagesResponse {
        conversation_id: conversation.id,
        messages,
    }))
}

#[utoipa::path(
    post,
    path = "/coach/plants/{plant_id}/messages",
    request_body = SendCoachMessageRequest,
    responses(
        (status = 201, description = "Message sent and AI response received", body = CoachMessageResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Plant not found"),
        (status = 502, description = "AI service unavailable"),
    ),
    params(
        ("plant_id" = Uuid, Path, description = "Plant ID")
    ),
    security(
        ("session" = [])
    ),
    tag = "coach"
)]
async fn send_message(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path(plant_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<SendCoachMessageRequest>,
) -> Result<(StatusCode, Json<CoachMessageResponse>)> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    // Verify plant belongs to user
    let plant = db_plants::get_plant_by_id(&app_state.pool, plant_id).await?;
    if plant.user_id != user.id {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    let coach = app_state.coach.as_ref().ok_or(AppError::External {
        message: "Plant coach is not configured. Set PLANT_COACH_PROVIDER and API key environment variables.".to_string(),
    })?;

    let conversation =
        db_coach::get_or_create_conversation(&app_state.pool, &plant_id.to_string(), &user.id)
            .await
            .map_err(|e| AppError::Internal {
                message: e.to_string(),
            })?;

    // Insert user message
    let _user_msg = db_coach::insert_message(
        &app_state.pool,
        &conversation.id,
        "user",
        &payload.content,
        payload.image_url.as_deref(),
    )
    .await
    .map_err(|e| AppError::Internal {
        message: e.to_string(),
    })?;

    // Build LLM context
    let system_prompt = build_system_prompt(&plant);
    let mut llm_messages = vec![ChatMessage {
        role: "system".to_string(),
        content: vec![ContentPart::Text {
            text: system_prompt,
        }],
    }];

    // Add conversation history
    let history = db_coach::get_messages(&app_state.pool, &conversation.id)
        .await
        .map_err(|e| AppError::Internal {
            message: e.to_string(),
        })?;

    for msg in &history {
        let mut parts: Vec<ContentPart> = vec![ContentPart::Text {
            text: msg.content.clone(),
        }];
        if let Some(url) = &msg.image_url {
            parts.push(ContentPart::ImageUrl {
                image_url: crate::llm::ImageUrlContent { url: url.clone() },
            });
        }
        llm_messages.push(ChatMessage {
            role: msg.role.clone(),
            content: parts,
        });
    }

    // Call the LLM
    let response = coach
        .chat(llm_messages)
        .await
        .map_err(|e| AppError::External {
            message: format!("AI coach error: {e}"),
        })?;

    // Insert assistant message
    let assistant_msg = db_coach::insert_message(
        &app_state.pool,
        &conversation.id,
        "assistant",
        &response.text,
        None,
    )
    .await
    .map_err(|e| AppError::Internal {
        message: e.to_string(),
    })?;

    // Insert suggestions
    let mut suggestions = Vec::new();
    for s in &response.suggestions {
        let row = db_coach::insert_suggestion(
            &app_state.pool,
            &assistant_msg.id,
            &plant_id.to_string(),
            &s.suggestion_type,
            &s.payload,
        )
        .await
        .map_err(|e| AppError::Internal {
            message: e.to_string(),
        })?;

        suggestions.push(CoachSuggestion {
            id: row.id,
            suggestion_type: row.suggestion_type,
            payload: serde_json::from_str(&row.payload).unwrap_or_default(),
            status: row.status,
            applied_at: row.applied_at,
        });
    }

    let message = CoachMessage {
        id: assistant_msg.id,
        role: "assistant".to_string(),
        content: response.text,
        image_url: None,
        suggestions,
        created_at: assistant_msg.created_at,
    };

    Ok((StatusCode::CREATED, Json(CoachMessageResponse { message })))
}

#[utoipa::path(
    post,
    path = "/coach/suggestions/{suggestion_id}/accept",
    responses(
        (status = 200, description = "Suggestion accepted", body = CoachSuggestion),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Suggestion not found"),
    ),
    params(
        ("suggestion_id" = String, Path, description = "Suggestion ID")
    ),
    security(
        ("session" = [])
    ),
    tag = "coach"
)]
async fn accept_suggestion(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path(suggestion_id): Path<String>,
) -> Result<Json<CoachSuggestion>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let row = db_coach::update_suggestion_status(&app_state.pool, &suggestion_id, &user.id, "accepted")
        .await
        .map_err(|e| AppError::Internal {
            message: e.to_string(),
        })?
        .ok_or(AppError::NotFound {
            resource: format!("Suggestion with id {suggestion_id}"),
        })?;

    Ok(Json(CoachSuggestion {
        id: row.id,
        suggestion_type: row.suggestion_type,
        payload: serde_json::from_str(&row.payload).unwrap_or_default(),
        status: row.status,
        applied_at: row.applied_at,
    }))
}

#[utoipa::path(
    post,
    path = "/coach/suggestions/{suggestion_id}/dismiss",
    responses(
        (status = 200, description = "Suggestion dismissed", body = CoachSuggestion),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Suggestion not found"),
    ),
    params(
        ("suggestion_id" = String, Path, description = "Suggestion ID")
    ),
    security(
        ("session" = [])
    ),
    tag = "coach"
)]
async fn dismiss_suggestion(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path(suggestion_id): Path<String>,
) -> Result<Json<CoachSuggestion>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let row = db_coach::update_suggestion_status(&app_state.pool, &suggestion_id, &user.id, "dismissed")
        .await
        .map_err(|e| AppError::Internal {
            message: e.to_string(),
        })?
        .ok_or(AppError::NotFound {
            resource: format!("Suggestion with id {suggestion_id}"),
        })?;

    Ok(Json(CoachSuggestion {
        id: row.id,
        suggestion_type: row.suggestion_type,
        payload: serde_json::from_str(&row.payload).unwrap_or_default(),
        status: row.status,
        applied_at: row.applied_at,
    }))
}

fn build_system_prompt(plant: &crate::models::PlantResponse) -> String {
    let care_tasks_info = if plant.care_tasks.is_empty() {
        "No care tasks configured yet.".to_string()
    } else {
        plant
            .care_tasks
            .iter()
            .map(|t| {
                let schedule = t.task.interval_days
                    .map(|d| format!("every {} days", d))
                    .unwrap_or_else(|| "manual/one-off".to_string());
                let last = t.task.last_performed
                    .map(|d| d.to_rfc3339())
                    .unwrap_or_else(|| "never".to_string());
                let due_info = if t.is_due {
                    format!(" [OVERDUE by {} days]", t.days_overdue.unwrap_or(0))
                } else if let Some(days) = t.days_overdue {
                    format!(" [due in {} days]", -days)
                } else {
                    String::new()
                };
                format!("  - {} ({}): schedule={}, last={}{}", 
                    t.task.name, 
                    t.task.icon.as_deref().unwrap_or(""),
                    schedule, last, due_info)
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "You are an expert plant care coach. You help users take care of their plants by analyzing photos, \
         diagnosing issues, and suggesting care plan adjustments. Always respond with JSON containing 'text' \
         (your natural language response) and 'suggestions' (array of actionable suggestions). Each suggestion \
         has 'suggestion_type' (one of: schedule_change, reminder, care_action, photo_request), 'description' \
         (human-readable), and 'payload' (structured data).\n\n\
         Plant context:\n\
         - Name: {}\n\
         - Genus: {}\n\
         - Care tasks:\n{}",
        plant.name, plant.genus, care_tasks_info
    )
}
