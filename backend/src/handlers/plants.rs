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

use crate::auth::AuthSession;
use crate::database::{plants as db_plants, DatabasePool};
use crate::handlers::{photos, tracking};
use crate::middleware::validation::ValidatedJson;
use crate::models::{CreatePlantRequest, PlantResponse, PlantsResponse, UpdatePlantRequest};
use crate::utils::errors::{AppError, Result};

pub fn routes() -> Router<DatabasePool> {
    Router::new()
        .route("/", get(list_plants).post(create_plant))
        .route(
            "/:id",
            get(get_plant).put(update_plant).delete(delete_plant),
        )
        .nest("/:plant_id", photos::routes())
        .merge(tracking::routes())
}

#[derive(Debug, Deserialize)]
struct ListPlantsQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<String>,
}

async fn list_plants(
    auth_session: AuthSession,
    State(pool): State<DatabasePool>,
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
        db_plants::list_plants_for_user(&pool, &user.id, limit, offset, params.search.as_deref())
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
    State(pool): State<DatabasePool>,
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

    let plant = db_plants::create_plant(&pool, &user.id, &payload).await?;

    tracing::info!("Created plant with id: {} for user: {}", plant.id, user.id);
    Ok((StatusCode::CREATED, Json(plant)))
}

async fn get_plant(
    auth_session: AuthSession,
    State(pool): State<DatabasePool>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlantResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!("Get plant request for id: {} by user: {}", id, user.id);

    let plant = db_plants::get_plant_by_id(&pool, id).await?;

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
    State(pool): State<DatabasePool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePlantRequest>,
) -> Result<Json<PlantResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!("Update plant request for id: {} by user: {}", id, user.id);
    tracing::debug!("Update payload: {:?}", payload);

    let plant = db_plants::update_plant(&pool, id, &user.id, &payload).await?;

    tracing::info!("Updated plant: {} for user: {}", plant.name, user.id);
    Ok(Json(plant))
}

async fn delete_plant(
    auth_session: AuthSession,
    State(pool): State<DatabasePool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!("Delete plant request for id: {} by user: {}", id, user.id);

    db_plants::delete_plant(&pool, id, &user.id).await?;

    tracing::info!("Deleted plant with id: {} for user: {}", id, user.id);
    Ok(StatusCode::NO_CONTENT)
}
