use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::AuthSession;
use crate::database::memory as db;
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
    }
}
