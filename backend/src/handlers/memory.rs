use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::Deserialize;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::AuthSession;
use crate::database::memory as db;
use crate::database::photos as db_photos;
use crate::llm::{ChatMessage, ContentPart, ImageUrlContent};
use crate::models::memory::{
    CreateMemoryRequest, HealthHearts, MemorySource, PlantMemoriesResponse, PlantMemory,
    UpdateMemoryRequest,
};
use crate::utils::errors::AppError;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/plants/:plant_id/memories",
            get(list_memories).post(create_memory),
        )
        .route(
            "/plants/:plant_id/memories/:memory_id",
            put(update_memory).delete(delete_memory),
        )
        .route("/plants/:plant_id/health", get(get_health))
        .route("/plants/:plant_id/assess-health", post(assess_health))
}

#[utoipa::path(
    get,
    path = "/coach/plants/{plant_id}/memories",
    params(("plant_id" = Uuid, Path, description = "Plant ID")),
    responses(
        (status = 200, description = "Plant memories", body = PlantMemoriesResponse),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "coach"
)]
async fn list_memories(
    State(app_state): State<AppState>,
    auth_session: AuthSession,
    Path(plant_id): Path<Uuid>,
) -> Result<Json<PlantMemoriesResponse>, AppError> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let response = db::list_memories_for_plant(&app_state.pool, &plant_id, &user.id).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/coach/plants/{plant_id}/memories",
    params(("plant_id" = Uuid, Path, description = "Plant ID")),
    request_body = CreateMemoryRequest,
    responses(
        (status = 201, description = "Memory created", body = PlantMemory),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "coach"
)]
async fn create_memory(
    State(app_state): State<AppState>,
    auth_session: AuthSession,
    Path(plant_id): Path<Uuid>,
    Json(payload): Json<CreateMemoryRequest>,
) -> Result<(StatusCode, Json<PlantMemory>), AppError> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let memory = db::create_memory(
        &app_state.pool,
        &plant_id,
        &user.id,
        &payload.fact_type,
        &payload.content,
        1.0, // User-created facts have full confidence
        MemorySource::User,
        None,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(memory)))
}

#[utoipa::path(
    put,
    path = "/coach/plants/{plant_id}/memories/{memory_id}",
    params(
        ("plant_id" = Uuid, Path, description = "Plant ID"),
        ("memory_id" = Uuid, Path, description = "Memory ID"),
    ),
    request_body = UpdateMemoryRequest,
    responses(
        (status = 200, description = "Memory updated", body = PlantMemory),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    ),
    tag = "coach"
)]
async fn update_memory(
    State(app_state): State<AppState>,
    auth_session: AuthSession,
    Path((_plant_id, memory_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateMemoryRequest>,
) -> Result<Json<PlantMemory>, AppError> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let memory = db::update_memory(
        &app_state.pool,
        &memory_id,
        &user.id,
        payload.content.as_deref(),
        payload.fact_type.as_ref(),
    )
    .await?;

    Ok(Json(memory))
}

#[utoipa::path(
    delete,
    path = "/coach/plants/{plant_id}/memories/{memory_id}",
    params(
        ("plant_id" = Uuid, Path, description = "Plant ID"),
        ("memory_id" = Uuid, Path, description = "Memory ID"),
    ),
    responses(
        (status = 204, description = "Memory deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    ),
    tag = "coach"
)]
async fn delete_memory(
    State(app_state): State<AppState>,
    auth_session: AuthSession,
    Path((_plant_id, memory_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    db::delete_memory(&app_state.pool, &memory_id, &user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/coach/plants/{plant_id}/health",
    params(("plant_id" = Uuid, Path, description = "Plant ID")),
    responses(
        (status = 200, description = "Plant health score as hearts", body = HealthHearts),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "coach"
)]
async fn get_health(
    State(app_state): State<AppState>,
    auth_session: AuthSession,
    Path(plant_id): Path<Uuid>,
) -> Result<Json<HealthHearts>, AppError> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    // Get latest score, or compute a basic one from care task adherence
    let score = db::get_latest_health_score(&app_state.pool, &plant_id, &user.id).await?;

    match score {
        Some(s) => Ok(Json(s.to_hearts())),
        None => {
            // No score yet — compute a basic one from care adherence
            let hearts = compute_basic_health(&app_state.pool, &plant_id, &user.id).await;
            Ok(Json(hearts))
        }
    }
}

/// Compute a basic health score from care task adherence (no stored score yet)
async fn compute_basic_health(
    pool: &crate::database::DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
) -> HealthHearts {
    let tasks =
        crate::database::care_tasks::list_care_tasks_for_plant(pool, plant_id, user_id, false)
            .await
            .ok();

    let score = match tasks {
        Some(resp) if !resp.tasks.is_empty() => {
            let total = resp.tasks.len() as f64;
            let overdue = resp.tasks.iter().filter(|t| t.is_due).count() as f64;
            // Simple: 1.0 if nothing overdue, decreases with overdue ratio
            (1.0 - (overdue / total) * 0.6).max(0.2)
        }
        _ => 0.8, // No tasks = assume healthy (new plant)
    };

    let hearts = (score * 5.0 * 2.0).round() / 2.0;
    HealthHearts {
        hearts,
        score,
        scored_at: None,
        reasoning: None,
    }
}

// ─── Health Assessment (AI Vision) ──────────────────────────────────────────

const HEALTH_ASSESSMENT_PROMPT: &str = r#"You are an expert plant health assessor. Analyze this photo of a plant and evaluate its overall health condition.

EVALUATION CRITERIA:
- Leaf color: vibrant green = healthy, yellowing/browning/pale = issues
- Leaf condition: firm and turgid = healthy, wilting/drooping/curling = stress
- Spots/marks: none = healthy, fungal spots/pest damage/burns = issues
- Growth: compact and proportional = healthy, leggy/sparse = light stress
- Soil visible: moist appropriate = good, bone dry or waterlogged = concern
- Overall vigor: new growth visible = thriving, stagnant or declining = concern

SCORING GUIDE:
- 5: Excellent — plant is thriving, vibrant color, active growth, no issues
- 4: Good — healthy overall with very minor cosmetic issues (one yellow leaf, slight dust)
- 3: Fair — noticeable issues that need attention (several yellowing leaves, slight wilting, minor pest signs)
- 2: Poor — significant problems (widespread wilting, heavy pest infestation, major leaf loss)
- 1: Critical — plant is in serious decline (severe rot, mostly dead leaves, needs immediate intervention)

Respond with ONLY a JSON object, no other text."#;

/// AI response schema for health assessment
fn health_assessment_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "score": {
                "type": "integer",
                "minimum": 1,
                "maximum": 5,
                "description": "Health score from 1 (critical) to 5 (excellent)"
            },
            "reasoning": {
                "type": "string",
                "description": "Brief explanation (1-2 sentences) of what you observe that determines this score"
            },
            "observations": {
                "type": "array",
                "items": { "type": "string" },
                "description": "List of specific observations (e.g. 'yellowing lower leaves', 'new growth tips visible')"
            }
        },
        "required": ["score", "reasoning", "observations"]
    })
}

/// AI response from health assessment
#[derive(Debug, Deserialize)]
struct AiHealthAssessment {
    score: u8,
    reasoning: String,
    #[allow(dead_code)]
    observations: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/coach/plants/{plant_id}/assess-health",
    params(("plant_id" = Uuid, Path, description = "Plant ID")),
    responses(
        (status = 200, description = "Health assessment complete", body = HealthHearts),
        (status = 404, description = "No photos available for this plant"),
        (status = 401, description = "Unauthorized"),
        (status = 503, description = "AI service unavailable"),
    ),
    tag = "coach"
)]
async fn assess_health(
    State(app_state): State<AppState>,
    auth_session: AuthSession,
    Path(plant_id): Path<Uuid>,
) -> Result<Json<HealthHearts>, AppError> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    // Get plant info for context
    let plant = crate::database::plants::get_plant_by_id(&app_state.pool, plant_id).await?;

    // Verify ownership
    if plant.user_id != user.id {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    // Get latest photo data
    let photos_resp =
        db_photos::get_photos_for_plant_paginated(&app_state.pool, &plant_id, &user.id, Some(1), None, Some(true))
            .await?;

    let latest_photo = photos_resp.photos.first().ok_or(AppError::NotFound {
        resource: "No photos available for health assessment".to_string(),
    })?;

    // Get raw photo data and encode as base64 data URL
    let (photo_data, content_type) =
        db_photos::get_photo_data(&app_state.pool, &plant_id, &latest_photo.id, &user.id).await?;
    let base64_data = BASE64.encode(&photo_data);
    let data_url = format!("data:{};base64,{}", content_type, base64_data);

    // Resolve LLM coach (per-user settings take priority)
    let coach = crate::llm::resolve_coach_for_request(
        user.llm_base_url.as_deref(),
        user.llm_api_key.as_deref(),
        user.llm_model.as_deref(),
        app_state.coach.as_ref(),
    )
    .map_err(|msg| AppError::External { message: msg })?;

    // Build messages for the vision model
    let system_message = ChatMessage {
        role: "system".to_string(),
        content: vec![ContentPart::Text {
            text: HEALTH_ASSESSMENT_PROMPT.to_string(),
        }],
    };

    let user_content = vec![
        ContentPart::ImageUrl {
            image_url: ImageUrlContent {
                url: data_url,
            },
        },
        ContentPart::Text {
            text: format!(
                "Assess the health of this plant. It is a {} ({}).",
                plant.name, plant.genus
            ),
        },
    ];

    let user_message = ChatMessage {
        role: "user".to_string(),
        content: user_content,
    };

    info!(
        "Health assessment request for plant {} ({}) by user {}",
        plant.name, plant_id, user.id
    );

    // Call AI with structured response schema
    let raw_response = coach
        .chat_raw(
            vec![system_message, user_message],
            Some(health_assessment_schema()),
        )
        .await
        .map_err(|e| {
            error!("Health assessment AI call failed: {}", e);
            AppError::External {
                message: format!("AI health assessment failed: {e}"),
            }
        })?;

    // Parse AI response
    let assessment: AiHealthAssessment = serde_json::from_str(&raw_response).map_err(|e| {
        warn!(
            "Failed to parse health assessment response: {}. Raw: {}",
            e,
            &raw_response[..raw_response.len().min(500)]
        );
        AppError::External {
            message: "AI returned an invalid health assessment format".to_string(),
        }
    })?;

    // Clamp score to 1-5 range
    let clamped_score = assessment.score.clamp(1, 5);
    // Convert 1-5 integer to 0.0-1.0 float for storage
    let normalized_score = (clamped_score as f64 - 1.0) / 4.0;

    // Also compute care adherence for a blended score
    let care_score = compute_care_adherence(&app_state.pool, &plant_id, &user.id).await;

    // Blend: 70% AI vision, 30% care adherence
    let blended_score = normalized_score * 0.7 + care_score * 0.3;

    // Store the score
    let stored = db::store_health_score(
        &app_state.pool,
        &plant_id,
        &user.id,
        blended_score,
        Some(care_score),
        None,
        Some(normalized_score),
        Some(&assessment.reasoning),
    )
    .await?;

    info!(
        "Health assessment stored for plant {}: score={:.2} (AI={}, care={:.2}, blended={:.2})",
        plant_id, stored.score, clamped_score, care_score, blended_score
    );

    Ok(Json(stored.to_hearts()))
}

/// Compute care adherence score (0.0-1.0) from care task status
async fn compute_care_adherence(
    pool: &crate::database::DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
) -> f64 {
    let tasks =
        crate::database::care_tasks::list_care_tasks_for_plant(pool, plant_id, user_id, false)
            .await
            .ok();

    match tasks {
        Some(resp) if !resp.tasks.is_empty() => {
            let total = resp.tasks.len() as f64;
            let overdue = resp.tasks.iter().filter(|t| t.is_due).count() as f64;
            (1.0 - (overdue / total)).max(0.0)
        }
        _ => 1.0, // No tasks = assume perfect adherence
    }
}

// ─── Background Health Assessment (for photo upload hook) ────────────────────

/// Trigger a health assessment in the background after a photo upload.
/// This is fire-and-forget — failures are logged but don't affect the caller.
pub fn trigger_background_assessment(
    app_state: AppState,
    plant_id: Uuid,
    user_id: String,
    coach: std::sync::Arc<dyn crate::llm::PlantCoach>,
) {
    tokio::spawn(async move {
        if let Err(e) = run_background_assessment(&app_state, &plant_id, &user_id, coach.as_ref())
            .await
        {
            warn!(
                "Background health assessment failed for plant {}: {}",
                plant_id, e
            );
        }
    });
}

async fn run_background_assessment(
    app_state: &AppState,
    plant_id: &Uuid,
    user_id: &str,
    coach: &dyn crate::llm::PlantCoach,
) -> Result<(), AppError> {
    // Get plant info
    let plant = crate::database::plants::get_plant_by_id(&app_state.pool, *plant_id).await?;

    // Get latest photo
    let photos_resp =
        db_photos::get_photos_for_plant_paginated(&app_state.pool, plant_id, user_id, Some(1), None, Some(true))
            .await?;

    let latest_photo = photos_resp.photos.first().ok_or(AppError::NotFound {
        resource: "No photos".to_string(),
    })?;

    let (photo_data, content_type) =
        db_photos::get_photo_data(&app_state.pool, plant_id, &latest_photo.id, user_id).await?;
    let base64_data = BASE64.encode(&photo_data);
    let data_url = format!("data:{};base64,{}", content_type, base64_data);

    // Build messages
    let system_message = ChatMessage {
        role: "system".to_string(),
        content: vec![ContentPart::Text {
            text: HEALTH_ASSESSMENT_PROMPT.to_string(),
        }],
    };

    let user_message = ChatMessage {
        role: "user".to_string(),
        content: vec![
            ContentPart::ImageUrl {
                image_url: ImageUrlContent { url: data_url },
            },
            ContentPart::Text {
                text: format!(
                    "Assess the health of this plant. It is a {} ({}).",
                    plant.name, plant.genus
                ),
            },
        ],
    };

    let raw_response = coach
        .chat_raw(
            vec![system_message, user_message],
            Some(health_assessment_schema()),
        )
        .await
        .map_err(|e| AppError::External {
            message: format!("AI call failed: {e}"),
        })?;

    let assessment: AiHealthAssessment =
        serde_json::from_str(&raw_response).map_err(|e| AppError::External {
            message: format!("Parse failed: {e}"),
        })?;

    let clamped_score = assessment.score.clamp(1, 5);
    let normalized_score = (clamped_score as f64 - 1.0) / 4.0;
    let care_score = compute_care_adherence(&app_state.pool, plant_id, user_id).await;
    let blended_score = normalized_score * 0.7 + care_score * 0.3;

    db::store_health_score(
        &app_state.pool,
        plant_id,
        user_id,
        blended_score,
        Some(care_score),
        None,
        Some(normalized_score),
        Some(&assessment.reasoning),
    )
    .await?;

    info!(
        "Background health assessment complete for plant {}: score={}",
        plant_id, clamped_score
    );

    Ok(())
}
