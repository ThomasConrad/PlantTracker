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
pub async fn get_messages(
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
                description: s.description,
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
pub async fn send_message(
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
    let system_prompt = format!(
        "{}{}",
        crate::llm::COACH_SYSTEM_PROMPT,
        crate::llm::build_plant_context(&plant)
    );
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
            &s.description,
            &s.payload,
        )
        .await
        .map_err(|e| AppError::Internal {
            message: e.to_string(),
        })?;

        suggestions.push(CoachSuggestion {
            id: row.id,
            suggestion_type: row.suggestion_type,
            description: row.description,
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
pub async fn accept_suggestion(
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

    // Apply the suggestion based on type
    let payload: serde_json::Value = serde_json::from_str(&row.payload).unwrap_or_default();
    let plant_id = &row.plant_id;

    match row.suggestion_type.as_str() {
        "schedule_change" => {
            // Update an existing care task's interval
            if let (Some(task_name), Some(interval_days)) = (
                payload.get("careTaskName").and_then(|v| v.as_str()),
                payload.get("intervalDays").and_then(|v| v.as_i64()),
            ) {
                let plant_uuid = uuid::Uuid::parse_str(plant_id).map_err(|_| AppError::Internal {
                    message: "Invalid plant_id".to_string(),
                })?;
                // Find the care task by name
                let tasks_resp = crate::database::care_tasks::list_care_tasks_for_plant(
                    &app_state.pool, &plant_uuid, &user.id, false
                ).await.map_err(|e| AppError::Internal { message: e.to_string() })?;

                if let Some(task) = tasks_resp.tasks.iter().find(|t| t.task.name.eq_ignore_ascii_case(task_name)) {
                    let update = crate::models::care_task::UpdateCareTaskRequest {
                        name: None,
                        icon: None,
                        color: None,
                        interval_days: Some(Some(interval_days as i32)),
                        amount: None,
                        unit: None,
                        notes: None,
                        last_performed: None,
                        sort_order: None,
                    };
                    let _ = crate::database::care_tasks::update_care_task(
                        &app_state.pool, &task.task.id, &user.id, &update
                    ).await;
                }
            }
        }
        "new_task" => {
            // Create a new care task
            if let Some(name) = payload.get("name").and_then(|v| v.as_str()) {
                let plant_uuid = uuid::Uuid::parse_str(plant_id).map_err(|_| AppError::Internal {
                    message: "Invalid plant_id".to_string(),
                })?;
                let create = crate::models::care_task::CreateCareTaskRequest {
                    name: name.to_string(),
                    icon: payload.get("icon").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    color: None,
                    interval_days: payload.get("intervalDays").and_then(|v| v.as_i64()).map(|d| d as i32),
                    amount: None,
                    unit: None,
                    notes: None,
                    last_performed: None,
                    sort_order: None,
                };
                let _ = crate::database::care_tasks::create_care_task(
                    &app_state.pool, &plant_uuid, &user.id, &create
                ).await;
            }
        }
        "care_action" => {
            // Log a care task completion
            if let Some(task_name) = payload.get("careTaskName").and_then(|v| v.as_str()) {
                let plant_uuid = uuid::Uuid::parse_str(plant_id).map_err(|_| AppError::Internal {
                    message: "Invalid plant_id".to_string(),
                })?;
                let tasks_resp = crate::database::care_tasks::list_care_tasks_for_plant(
                    &app_state.pool, &plant_uuid, &user.id, false
                ).await.map_err(|e| AppError::Internal { message: e.to_string() })?;

                if let Some(task) = tasks_resp.tasks.iter().find(|t| t.task.name.eq_ignore_ascii_case(task_name)) {
                    let log_req = crate::models::care_task::LogCareTaskRequest {
                        timestamp: None,
                        value: None,
                        notes: Some("Logged via coach suggestion".to_string()),
                        photo_ids: None,
                    };
                    let _ = crate::database::care_tasks::log_care_task(
                        &app_state.pool, &task.task.id, &user.id, &log_req
                    ).await;
                }
            }
        }
        // "photo_request" has no server-side action — the UI handles it
        _ => {}
    }

    Ok(Json(CoachSuggestion {
        id: row.id,
        suggestion_type: row.suggestion_type,
        description: row.description,
        payload,
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
pub async fn dismiss_suggestion(
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
        description: row.description,
        payload: serde_json::from_str(&row.payload).unwrap_or_default(),
        status: row.status,
        applied_at: row.applied_at,
    }))
}



