#[allow(unused_imports)]
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::AuthSession;
use crate::database::plants as db_plants;
use crate::handlers::{photos, tracking};
use crate::middleware::validation::ValidatedJson;
use crate::models::{CreatePlantRequest, PlantResponse, PlantsResponse, UpdatePlantRequest};
use crate::utils::errors::{AppError, Result};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_plants).post(create_plant))
        .route(
            "/:id",
            get(get_plant).put(update_plant).delete(delete_plant),
        )
        .route("/:id/thumbnail/:photo_id", put(set_plant_thumbnail))
        .nest("/:plant_id", photos::routes())
        .merge(tracking::routes())
}

#[derive(Debug, Deserialize)]
struct ListPlantsQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<String>,
    sort: Option<String>, // "date_asc", "date_desc" (default), "name_asc", "name_desc"
}

async fn list_plants(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Query(params): Query<ListPlantsQuery>,
) -> Result<Json<PlantsResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!(
        "List plants request for user {} with params: {:?}",
        user.id,
        params
    );

    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    let (plants, total) =
        db_plants::list_plants_for_user_with_sort(&app_state.pool, &user.id, limit, offset, params.search.as_deref(), params.sort.as_deref())
            .await?;

    let response = PlantsResponse {
        plants,
        total,
        limit,
        offset,
    };

    tracing::debug!(
        "Returning {} plants for user {}",
        response.plants.len(),
        user.id
    );
    Ok(Json(response))
}

async fn create_plant(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreatePlantRequest>,
) -> Result<(StatusCode, Json<PlantResponse>)> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!(
        "Create plant request for user {}: name={}, genus={}",
        user.id,
        payload.name,
        payload.genus
    );

    let plant = db_plants::create_plant(&app_state.pool, &user.id, &payload).await?;

    tracing::info!("Created plant with id: {} for user: {}", plant.id, user.id);
    Ok((StatusCode::CREATED, Json(plant)))
}

async fn get_plant(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlantResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!("Get plant request for id: {} by user: {}", id, user.id);

    let plant = db_plants::get_plant_by_id(&app_state.pool, id).await?;

    // Verify the plant belongs to the authenticated user
    if plant.user_id != user.id {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {id}"),
        });
    }

    tracing::debug!("Retrieved plant: {} for user: {}", plant.name, user.id);
    Ok(Json(plant))
}

async fn update_plant(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePlantRequest>,
) -> Result<Json<PlantResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!("Update plant request for id: {} by user: {}", id, user.id);
    tracing::debug!("Update payload: {:?}", payload);

    let plant = db_plants::update_plant(&app_state.pool, id, &user.id, &payload).await?;

    tracing::info!("Updated plant: {} for user: {}", plant.name, user.id);
    Ok(Json(plant))
}

async fn delete_plant(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!("Delete plant request for id: {} by user: {}", id, user.id);

    db_plants::delete_plant(&app_state.pool, id, &user.id).await?;

    tracing::info!("Deleted plant with id: {} for user: {}", id, user.id);
    Ok(StatusCode::NO_CONTENT)
}

async fn set_plant_thumbnail(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path((id, photo_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<PlantResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!(
        "Set thumbnail request for plant: {}, photo: {} by user: {}",
        id,
        photo_id,
        user.id
    );

    let plant = db_plants::set_plant_thumbnail(&app_state.pool, id, photo_id, &user.id).await?;

    tracing::info!(
        "Set thumbnail for plant: {} to photo: {} for user: {}",
        id,
        photo_id,
        user.id
    );
    Ok(Json(plant))
}
