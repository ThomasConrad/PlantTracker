use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, put},
    Router,
};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::database::plant_attributes as db;
use crate::database::plants as db_plants;
use crate::extractors::AuthenticatedUser;
use crate::middleware::validation::ValidatedJson;
use crate::models::plant_attribute::{
    CreatePlantAttributeRequest, PlantAttribute, PlantAttributesResponse,
    UpdatePlantAttributeRequest,
};
use crate::utils::errors::{AppError, Result};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/plants/:plant_id/attributes",
            get(list_attributes).post(create_attribute),
        )
        .route(
            "/plants/:plant_id/attributes/:attribute_id",
            put(update_attribute).delete(delete_attribute),
        )
}

// ─── List attributes for a plant ─────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/plants/{plant_id}/attributes",
    params(("plant_id" = Uuid, Path, description = "Plant ID")),
    responses(
        (status = 200, description = "Plant attributes", body = PlantAttributesResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Plant not found"),
    ),
    security(("session" = [])),
    tag = "plant_attributes"
)]
pub async fn list_attributes(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    Path(plant_id): Path<Uuid>,
) -> Result<Json<PlantAttributesResponse>> {
    // Verify plant belongs to user
    let plant = db_plants::get_plant_by_id(&app_state.pool, plant_id).await?;
    if plant.user_id != user.id {
        return Err(AppError::NotFound {
            resource: format!("Plant {plant_id}"),
        });
    }

    let response = db::list_attributes_for_plant(&app_state.pool, &plant_id, &user.id).await?;
    Ok(Json(response))
}

// ─── Create (upsert) an attribute ────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/plants/{plant_id}/attributes",
    params(("plant_id" = Uuid, Path, description = "Plant ID")),
    request_body = CreatePlantAttributeRequest,
    responses(
        (status = 201, description = "Attribute created/updated", body = PlantAttribute),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Plant not found"),
    ),
    security(("session" = [])),
    tag = "plant_attributes"
)]
pub async fn create_attribute(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    Path(plant_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<CreatePlantAttributeRequest>,
) -> Result<(StatusCode, Json<PlantAttribute>)> {
    // Verify plant belongs to user
    let plant = db_plants::get_plant_by_id(&app_state.pool, plant_id).await?;
    if plant.user_id != user.id {
        return Err(AppError::NotFound {
            resource: format!("Plant {plant_id}"),
        });
    }

    let attribute = db::upsert_attribute(&app_state.pool, &plant_id, &user.id, &payload).await?;
    Ok((StatusCode::CREATED, Json(attribute)))
}

// ─── Update an attribute ─────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/plants/{plant_id}/attributes/{attribute_id}",
    params(
        ("plant_id" = Uuid, Path, description = "Plant ID"),
        ("attribute_id" = Uuid, Path, description = "Attribute ID"),
    ),
    request_body = UpdatePlantAttributeRequest,
    responses(
        (status = 200, description = "Attribute updated", body = PlantAttribute),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Attribute not found"),
    ),
    security(("session" = [])),
    tag = "plant_attributes"
)]
pub async fn update_attribute(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    Path((plant_id, attribute_id)): Path<(Uuid, Uuid)>,
    ValidatedJson(payload): ValidatedJson<UpdatePlantAttributeRequest>,
) -> Result<Json<PlantAttribute>> {
    // Verify plant belongs to user
    let plant = db_plants::get_plant_by_id(&app_state.pool, plant_id).await?;
    if plant.user_id != user.id {
        return Err(AppError::NotFound {
            resource: format!("Plant {plant_id}"),
        });
    }

    let attribute = db::update_attribute(
        &app_state.pool,
        &attribute_id,
        &user.id,
        payload.label.as_deref(),
        payload.value.as_deref(),
        payload.icon.as_ref().map(|o| o.as_deref()),
        payload.category.as_ref().map(|o| o.as_deref()),
    )
    .await?;

    Ok(Json(attribute))
}

// ─── Delete an attribute ─────────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/plants/{plant_id}/attributes/{attribute_id}",
    params(
        ("plant_id" = Uuid, Path, description = "Plant ID"),
        ("attribute_id" = Uuid, Path, description = "Attribute ID"),
    ),
    responses(
        (status = 204, description = "Attribute deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Attribute not found"),
    ),
    security(("session" = [])),
    tag = "plant_attributes"
)]
pub async fn delete_attribute(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    Path((plant_id, attribute_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
    // Verify plant belongs to user
    let plant = db_plants::get_plant_by_id(&app_state.pool, plant_id).await?;
    if plant.user_id != user.id {
        return Err(AppError::NotFound {
            resource: format!("Plant {plant_id}"),
        });
    }

    db::delete_attribute(&app_state.pool, &attribute_id, &user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}
