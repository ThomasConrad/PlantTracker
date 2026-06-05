use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        Json,
    },
    routing::{get, post},
    Router,
};
use futures_util::stream::Stream;
use std::convert::Infallible;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::database::coach as db_coach;
use crate::database::plants as db_plants;
use crate::extractors::AuthenticatedUser;
use crate::llm::{ChatMessage, ContentPart};
use crate::middleware::validation::ValidatedJson;
use crate::models::coach::*;
use crate::utils::errors::{AppError, Result};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/plants/:plant_id/messages",
            get(get_messages).post(send_message),
        )
        .route(
            "/plants/:plant_id/messages/stream",
            post(stream_message),
        )
        .route(
            "/plants/:plant_id/suggestions",
            get(get_plant_suggestions),
        )
        .route("/suggestions/pending", get(get_all_pending_suggestions))
        .route(
            "/suggestions/:suggestion_id/accept",
            post(accept_suggestion),
        )
        .route(
            "/suggestions/:suggestion_id/dismiss",
            post(dismiss_suggestion),
        )
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
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    Path(plant_id): Path<Uuid>,
) -> Result<Json<CoachMessagesResponse>> {

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
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    Path(plant_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<SendCoachMessageRequest>,
) -> Result<(StatusCode, Json<CoachMessageResponse>)> {

    // At least one of content or image must be provided
    if payload.content.is_empty() && payload.image_url.is_none() {
        return Err(AppError::Parse {
            message: "Message must contain text or an image".to_string(),
        });
    }

    // Verify plant belongs to user
    let plant = db_plants::get_plant_by_id(&app_state.pool, plant_id).await?;
    if plant.user_id != user.id {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    // Resolve coach: per-user LLM settings take priority over global config
    let coach = user.resolve_coach(app_state.coach.as_ref())?;
    let coach_ref: &dyn crate::llm::PlantCoach = coach.as_ref();

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
    let memory_context =
        crate::database::memory::get_memory_context(&app_state.pool, &plant_id, &user.id)
            .await
            .unwrap_or_default();
    let system_prompt = format!(
        "{}{}{}",
        crate::llm::COACH_SYSTEM_PROMPT,
        crate::llm::build_plant_context(&plant),
        memory_context
    );
    let mut llm_messages = vec![ChatMessage {
        role: "system".to_string(),
        content: vec![ContentPart::Text {
            text: system_prompt,
        }],
    }];

    // Add conversation history
    // Only send the actual image data for the LATEST user message (the one just sent).
    // For older messages that had images, insert a text placeholder — the assistant's
    // subsequent response already contains the annotation/analysis of that photo,
    // which serves as persistent context without re-sending expensive image data.
    let history = db_coach::get_messages(&app_state.pool, &conversation.id)
        .await
        .map_err(|e| AppError::Internal {
            message: e.to_string(),
        })?;

    let last_idx = history.len().saturating_sub(1);
    for (i, msg) in history.iter().enumerate() {
        let mut parts: Vec<ContentPart> = vec![ContentPart::Text {
            text: msg.content.clone(),
        }];
        if let Some(url) = &msg.image_url {
            if i == last_idx {
                // Latest message: send the actual image for vision analysis
                parts.push(ContentPart::ImageUrl {
                    image_url: crate::llm::ImageUrlContent { url: url.clone() },
                });
            } else {
                // Older message: placeholder — the assistant's response has the analysis
                parts.push(ContentPart::Text {
                    text: "[User attached a photo — see assistant's analysis in the next message]"
                        .to_string(),
                });
            }
        }
        llm_messages.push(ChatMessage {
            role: msg.role.clone(),
            content: parts,
        });
    }

    // Call the LLM
    let response = coach_ref
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

    // Store extracted facts as plant memories
    if !response.extracted_facts.is_empty() {
        let facts: Vec<crate::models::memory::ExtractedFact> = response
            .extracted_facts
            .iter()
            .map(|f| crate::models::memory::ExtractedFact {
                fact_type: f.fact_type.clone(),
                content: f.content.clone(),
                confidence: f.confidence,
            })
            .collect();
        let _ = crate::database::memory::store_extracted_facts(
            &app_state.pool,
            &plant_id,
            &user.id,
            &facts,
            Some(&assistant_msg.id),
        )
        .await;
    }

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

    // Fire push notification for new actionable suggestions
    if !suggestions.is_empty() {
        notify_new_suggestions(
            app_state.pool.clone(),
            user.id.clone(),
            plant.name.clone(),
            suggestions.clone(),
        );
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

// ─── Suggestion Listing Endpoints ───────────────────────────────────────────

/// Response for pending suggestions, includes plant name for dashboard display
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PendingSuggestion {
    pub id: String,
    pub plant_id: String,
    pub plant_name: String,
    pub suggestion_type: String,
    pub description: String,
    pub payload: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PendingSuggestionsResponse {
    pub suggestions: Vec<PendingSuggestion>,
}

#[utoipa::path(
    get,
    path = "/coach/suggestions/pending",
    responses(
        (status = 200, description = "All pending suggestions for user", body = PendingSuggestionsResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(("session" = [])),
    tag = "coach"
)]
async fn get_all_pending_suggestions(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
) -> Result<Json<PendingSuggestionsResponse>> {

    let rows = db_coach::get_all_pending_suggestions_for_user(&app_state.pool, &user.id)
        .await
        .map_err(|e| AppError::Internal {
            message: e.to_string(),
        })?;

    // Fetch plant names for each unique plant_id
    let mut suggestions = Vec::new();
    for row in rows {
        let plant_name = match Uuid::parse_str(&row.plant_id) {
            Ok(pid) => db_plants::get_plant_by_id(&app_state.pool, pid)
                .await
                .map(|p| p.name)
                .unwrap_or_else(|_| "Unknown plant".to_string()),
            Err(_) => "Unknown plant".to_string(),
        };

        suggestions.push(PendingSuggestion {
            id: row.id,
            plant_id: row.plant_id,
            plant_name,
            suggestion_type: row.suggestion_type,
            description: row.description,
            payload: serde_json::from_str(&row.payload).unwrap_or_default(),
            created_at: row.created_at,
        });
    }

    Ok(Json(PendingSuggestionsResponse { suggestions }))
}

#[utoipa::path(
    get,
    path = "/coach/plants/{plant_id}/suggestions",
    responses(
        (status = 200, description = "Pending suggestions for plant", body = PendingSuggestionsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Plant not found"),
    ),
    params(("plant_id" = Uuid, Path, description = "Plant ID")),
    security(("session" = [])),
    tag = "coach"
)]
async fn get_plant_suggestions(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    Path(plant_id): Path<Uuid>,
) -> Result<Json<PendingSuggestionsResponse>> {

    // Verify plant ownership
    let plant = db_plants::get_plant_by_id(&app_state.pool, plant_id).await?;
    if plant.user_id != user.id {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    let rows =
        db_coach::get_pending_suggestions(&app_state.pool, &plant_id.to_string(), &user.id)
            .await
            .map_err(|e| AppError::Internal {
                message: e.to_string(),
            })?;

    let suggestions: Vec<PendingSuggestion> = rows
        .into_iter()
        .map(|row| PendingSuggestion {
            id: row.id,
            plant_id: row.plant_id,
            plant_name: plant.name.clone(),
            suggestion_type: row.suggestion_type,
            description: row.description,
            payload: serde_json::from_str(&row.payload).unwrap_or_default(),
            created_at: row.created_at,
        })
        .collect();

    Ok(Json(PendingSuggestionsResponse { suggestions }))
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
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    Path(suggestion_id): Path<String>,
) -> Result<Json<CoachSuggestion>> {

    let row =
        db_coach::update_suggestion_status(&app_state.pool, &suggestion_id, &user.id, "accepted")
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
                let plant_uuid =
                    uuid::Uuid::parse_str(plant_id).map_err(|_| AppError::Internal {
                        message: "Invalid plant_id".to_string(),
                    })?;
                // Find the care task by name
                let tasks_resp = crate::database::care_tasks::list_care_tasks_for_plant(
                    &app_state.pool,
                    &plant_uuid,
                    &user.id,
                    false,
                )
                .await
                .map_err(|e| AppError::Internal {
                    message: e.to_string(),
                })?;

                if let Some(task) = tasks_resp
                    .tasks
                    .iter()
                    .find(|t| t.task.name.eq_ignore_ascii_case(task_name))
                {
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
                        &app_state.pool,
                        &task.task.id,
                        &user.id,
                        &update,
                    )
                    .await;
                }
            }
        }
        "new_task" => {
            // Create a new care task
            if let Some(name) = payload.get("name").and_then(|v| v.as_str()) {
                let plant_uuid =
                    uuid::Uuid::parse_str(plant_id).map_err(|_| AppError::Internal {
                        message: "Invalid plant_id".to_string(),
                    })?;
                let create = crate::models::care_task::CreateCareTaskRequest {
                    name: name.to_string(),
                    icon: payload
                        .get("icon")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    color: None,
                    interval_days: payload
                        .get("intervalDays")
                        .and_then(|v| v.as_i64())
                        .map(|d| d as i32),
                    amount: None,
                    unit: None,
                    notes: None,
                    last_performed: None,
                    sort_order: None,
                };
                let _ = crate::database::care_tasks::create_care_task(
                    &app_state.pool,
                    &plant_uuid,
                    &user.id,
                    &create,
                )
                .await;
            }
        }
        "care_action" => {
            // Log a care task completion
            if let Some(task_name) = payload.get("careTaskName").and_then(|v| v.as_str()) {
                let plant_uuid =
                    uuid::Uuid::parse_str(plant_id).map_err(|_| AppError::Internal {
                        message: "Invalid plant_id".to_string(),
                    })?;
                let tasks_resp = crate::database::care_tasks::list_care_tasks_for_plant(
                    &app_state.pool,
                    &plant_uuid,
                    &user.id,
                    false,
                )
                .await
                .map_err(|e| AppError::Internal {
                    message: e.to_string(),
                })?;

                if let Some(task) = tasks_resp
                    .tasks
                    .iter()
                    .find(|t| t.task.name.eq_ignore_ascii_case(task_name))
                {
                    let log_req = crate::models::care_task::LogCareTaskRequest {
                        timestamp: None,
                        value: None,
                        notes: Some("Logged via coach suggestion".to_string()),
                        photo_ids: None,
                    };
                    let _ = crate::database::care_tasks::log_care_task(
                        &app_state.pool,
                        &task.task.id,
                        &user.id,
                        &log_req,
                    )
                    .await;
                }
            }
        }
        "photo_request" => {
            // Create a "Photo check-in" care task if one doesn't already exist
            let plant_uuid = uuid::Uuid::parse_str(plant_id).map_err(|_| AppError::Internal {
                message: "Invalid plant_id".to_string(),
            })?;
            let tasks_resp = crate::database::care_tasks::list_care_tasks_for_plant(
                &app_state.pool,
                &plant_uuid,
                &user.id,
                false,
            )
            .await
            .map_err(|e| AppError::Internal {
                message: e.to_string(),
            })?;

            let has_photo_task = tasks_resp.tasks.iter().any(|t| {
                let name_lower = t.task.name.to_lowercase();
                name_lower.contains("photo")
                    && (name_lower.contains("check")
                        || name_lower.contains("update")
                        || name_lower.contains("progress"))
            });

            if !has_photo_task {
                let interval = payload
                    .get("intervalDays")
                    .and_then(|v| v.as_i64())
                    .map(|d| d as i32)
                    .unwrap_or(14);
                let create = crate::models::care_task::CreateCareTaskRequest {
                    name: "Photo check-in".to_string(),
                    icon: Some("📸".to_string()),
                    color: None,
                    interval_days: Some(interval),
                    amount: None,
                    unit: None,
                    notes: Some("Take a photo to track growth progress".to_string()),
                    last_performed: None,
                    sort_order: None,
                };
                let _ = crate::database::care_tasks::create_care_task(
                    &app_state.pool,
                    &plant_uuid,
                    &user.id,
                    &create,
                )
                .await;
            }
        }
        "species_correction" => {
            // Update the plant's genus (species stored as memory note)
            let plant_uuid = uuid::Uuid::parse_str(plant_id).map_err(|_| AppError::Internal {
                message: "Invalid plant_id".to_string(),
            })?;
            let genus = payload
                .get("genus")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            if let Some(genus) = genus {
                let update = crate::models::UpdatePlantRequest {
                    name: None,
                    genus: Some(genus),
                    custom_metrics: None,
                };
                let _ = crate::database::plants::update_plant(
                    &app_state.pool,
                    plant_uuid,
                    &user.id,
                    &update,
                )
                .await;
            }
            // Store species as a memory note if provided
            if let Some(species) = payload.get("species").and_then(|v| v.as_str()) {
                let plant_uuid =
                    uuid::Uuid::parse_str(plant_id).map_err(|_| AppError::Internal {
                        message: "Invalid plant_id".to_string(),
                    })?;
                let _ = crate::database::memory::create_memory(
                    &app_state.pool,
                    &plant_uuid,
                    &user.id,
                    &crate::models::memory::MemoryFactType::SpeciesNote,
                    &format!("Species: {species}"),
                    0.9,
                    crate::models::memory::MemorySource::Coach,
                    None,
                )
                .await;
            }
        }
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
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    Path(suggestion_id): Path<String>,
) -> Result<Json<CoachSuggestion>> {

    let row =
        db_coach::update_suggestion_status(&app_state.pool, &suggestion_id, &user.id, "dismissed")
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

/// SSE streaming endpoint for coach messages.
/// Streams text tokens as `event: token` and sends the final structured response as `event: done`.
/// On error, sends `event: error` with the error message.
pub async fn stream_message(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    Path(plant_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<SendCoachMessageRequest>,
) -> Result<Sse<impl Stream<Item = std::result::Result<Event, Infallible>>>> {

    // At least one of content or image must be provided
    if payload.content.is_empty() && payload.image_url.is_none() {
        return Err(AppError::Parse {
            message: "Message must contain text or an image".to_string(),
        });
    }

    // Verify plant belongs to user
    let plant = db_plants::get_plant_by_id(&app_state.pool, plant_id).await?;
    if plant.user_id != user.id {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    // Resolve coach
    let coach = user.resolve_coach(app_state.coach.as_ref())?;

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

    // Build LLM messages
    let memory_context =
        crate::database::memory::get_memory_context(&app_state.pool, &plant_id, &user.id)
            .await
            .unwrap_or_default();
    let system_prompt = format!(
        "{}{}{}",
        crate::llm::COACH_SYSTEM_PROMPT,
        crate::llm::build_plant_context(&plant),
        memory_context
    );
    let mut llm_messages = vec![ChatMessage {
        role: "system".to_string(),
        content: vec![ContentPart::Text {
            text: system_prompt,
        }],
    }];

    let history = db_coach::get_messages(&app_state.pool, &conversation.id)
        .await
        .map_err(|e| AppError::Internal {
            message: e.to_string(),
        })?;

    let last_idx = history.len().saturating_sub(1);
    for (i, msg) in history.iter().enumerate() {
        let mut parts: Vec<ContentPart> = vec![ContentPart::Text {
            text: msg.content.clone(),
        }];
        if let Some(url) = &msg.image_url {
            if i == last_idx {
                parts.push(ContentPart::ImageUrl {
                    image_url: crate::llm::ImageUrlContent { url: url.clone() },
                });
            } else {
                parts.push(ContentPart::Text {
                    text: "[User attached a photo — see assistant's analysis in the next message]"
                        .to_string(),
                });
            }
        }
        llm_messages.push(ChatMessage {
            role: msg.role.clone(),
            content: parts,
        });
    }

    // Single channel for all SSE events (tokens, done, error)
    let (event_tx, event_rx) = tokio::sync::mpsc::channel::<StreamEvent>(64);

    let pool = app_state.pool.clone();
    let conv_id = conversation.id.clone();
    let user_id = user.id.clone();
    let plant_name = plant.name.clone();

    tokio::spawn(async move {
        // Token channel: provider sends text chunks here
        let (token_tx, mut token_rx) = tokio::sync::mpsc::channel::<String>(64);

        let event_tx_fwd = event_tx.clone();
        // Forward tokens → SSE events
        let forwarder = tokio::spawn(async move {
            while let Some(token) = token_rx.recv().await {
                if event_tx_fwd.send(StreamEvent::Token(token)).await.is_err() {
                    break;
                }
            }
        });

        let result = coach.stream_chat(llm_messages, token_tx).await;
        // Wait for all tokens to be forwarded before sending done/error
        forwarder.await.ok();

        match result {
            Ok(response) => {
                // Persist assistant message
                let assistant_msg = db_coach::insert_message(
                    &pool,
                    &conv_id,
                    "assistant",
                    &response.text,
                    None,
                )
                .await;

                if let Ok(assistant_msg) = assistant_msg {
                    // Store extracted facts
                    if !response.extracted_facts.is_empty() {
                        let facts: Vec<crate::models::memory::ExtractedFact> = response
                            .extracted_facts
                            .iter()
                            .map(|f| crate::models::memory::ExtractedFact {
                                fact_type: f.fact_type.clone(),
                                content: f.content.clone(),
                                confidence: f.confidence,
                            })
                            .collect();
                        let _ = crate::database::memory::store_extracted_facts(
                            &pool,
                            &plant_id,
                            &user_id,
                            &facts,
                            Some(&assistant_msg.id),
                        )
                        .await;
                    }

                    // Insert suggestions
                    let mut suggestions = Vec::new();
                    for s in &response.suggestions {
                        if let Ok(row) = db_coach::insert_suggestion(
                            &pool,
                            &assistant_msg.id,
                            &plant_id.to_string(),
                            &s.suggestion_type,
                            &s.description,
                            &s.payload,
                        )
                        .await
                        {
                            suggestions.push(CoachSuggestion {
                                id: row.id,
                                suggestion_type: row.suggestion_type,
                                description: row.description,
                                payload: serde_json::from_str(&row.payload).unwrap_or_default(),
                                status: row.status,
                                applied_at: row.applied_at,
                            });
                        }
                    }

                    // Fire push notification for new actionable suggestions
                    if !suggestions.is_empty() {
                        notify_new_suggestions(
                            pool.clone(),
                            user_id.clone(),
                            plant_name.clone(),
                            suggestions.clone(),
                        );
                    }

                    let message = CoachMessage {
                        id: assistant_msg.id,
                        role: "assistant".to_string(),
                        content: response.text,
                        image_url: None,
                        suggestions,
                        created_at: assistant_msg.created_at,
                    };
                    let _ = event_tx.send(StreamEvent::Done(message)).await;
                } else {
                    let _ = event_tx
                        .send(StreamEvent::Error("Failed to store message".to_string()))
                        .await;
                }
            }
            Err(e) => {
                let _ = event_tx
                    .send(StreamEvent::Error(format!("AI coach error: {e}")))
                    .await;
            }
        }
    });

    let stream = futures_util::stream::unfold(event_rx, |mut rx| async move {
        match rx.recv().await {
            Some(StreamEvent::Token(text)) => {
                let event = Event::default().event("token").data(text);
                Some((Ok(event), rx))
            }
            Some(StreamEvent::Done(message)) => {
                let json = serde_json::to_string(&message).unwrap_or_default();
                let event = Event::default().event("done").data(json);
                Some((Ok(event), rx))
            }
            Some(StreamEvent::Error(msg)) => {
                let event = Event::default().event("error").data(msg);
                Some((Ok(event), rx))
            }
            None => None,
        }
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

#[derive(Debug)]
enum StreamEvent {
    Token(String),
    Done(CoachMessage),
    Error(String),
}

/// Fire-and-forget: send push notifications for new coach suggestions.
/// Only sends for actionable suggestion types (schedule_change, care_action, new_task).
fn notify_new_suggestions(
    pool: crate::database::DatabasePool,
    user_id: String,
    plant_name: String,
    suggestions: Vec<CoachSuggestion>,
) {
    use crate::utils::push::{PushCategory, PushPayload, send_push_if_allowed};

    // Only notify for actionable suggestions
    let actionable: Vec<CoachSuggestion> = suggestions
        .into_iter()
        .filter(|s| matches!(s.suggestion_type.as_str(), "schedule_change" | "care_action" | "new_task"))
        .collect();

    if actionable.is_empty() {
        return;
    }

    tokio::spawn(async move {
        let (title, body) = if actionable.len() == 1 {
            let s = &actionable[0];
            (
                format!("Suggestion for {}", plant_name),
                s.description.clone(),
            )
        } else {
            (
                format!("{} suggestions for {}", actionable.len(), plant_name),
                actionable
                    .iter()
                    .take(2)
                    .map(|s| s.description.as_str())
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        };

        let payload = PushPayload {
            title,
            body,
            url: Some("/plants".to_string()),
            icon: None,
            tag: Some(format!("coach-suggestion-{}", plant_name.to_lowercase().replace(' ', "-"))),
        };

        send_push_if_allowed(&pool, &user_id, PushCategory::CoachSuggestion, &payload).await;
    });
}
