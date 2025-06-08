use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::middleware::validation::ValidatedJson;
use crate::models::{CreatePlantRequest, PlantResponse, PlantsResponse, UpdatePlantRequest};
use crate::utils::errors::{AppError, Result};

pub fn routes() -> Router {
    Router::new()
        .route("/", get(list_plants).post(create_plant))
        .route(
            "/:id",
            get(get_plant).put(update_plant).delete(delete_plant),
        )
}

#[derive(Debug, Deserialize)]
struct ListPlantsQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<String>,
}

async fn list_plants(
    Query(params): Query<ListPlantsQuery>,
) -> Result<Json<PlantsResponse>> {
    tracing::info!("List plants request with params: {:?}", params);

    // TODO: Implement actual plant listing
    // Mock response for now
    let response = PlantsResponse {
        plants: vec![],
        total: 0,
        limit: params.limit.unwrap_or(20),
        offset: params.offset.unwrap_or(0),
    };

    tracing::debug!("Returning {} plants", response.plants.len());
    Ok(Json(response))
}

async fn create_plant(
    ValidatedJson(payload): ValidatedJson<CreatePlantRequest>,
) -> Result<Json<PlantResponse>> {
    tracing::info!("Create plant request: name={}, genus={}", payload.name, payload.genus);

    // TODO: Implement actual plant creation
    // Mock response for now
    let mock_plant = PlantResponse {
        id: Uuid::new_v4(),
        name: payload.name.clone(),
        genus: payload.genus.clone(),
        watering_interval_days: payload.watering_interval_days,
        fertilizing_interval_days: payload.fertilizing_interval_days,
        last_watered: None,
        last_fertilized: None,
        custom_metrics: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        user_id: Uuid::new_v4(),
    };

    tracing::info!("Created plant with id: {}", mock_plant.id);
    Ok(Json(mock_plant))
}

async fn get_plant(Path(id): Path<Uuid>) -> Result<Json<PlantResponse>> {
    tracing::info!("Get plant request for id: {}", id);

    // TODO: Implement actual plant retrieval
    // For now, return a mock plant or 404 if not found
    // In real implementation, would query database and return AppError::NotFound if not exists
    
    let mock_plant = PlantResponse {
        id,
        name: "Mock Plant".to_string(),
        genus: "Mock Genus".to_string(),
        watering_interval_days: 7,
        fertilizing_interval_days: 14,
        last_watered: None,
        last_fertilized: None,
        custom_metrics: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        user_id: Uuid::new_v4(),
    };

    tracing::debug!("Retrieved plant: {}", mock_plant.name);
    Ok(Json(mock_plant))
}

async fn update_plant(
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePlantRequest>,
) -> Result<Json<PlantResponse>> {
    tracing::info!("Update plant request for id: {}", id);
    tracing::debug!("Update payload: {:?}", payload);

    // TODO: Implement actual plant update
    // In real implementation, would:
    // 1. Check if plant exists (return AppError::NotFound if not)
    // 2. Validate updated fields
    // 3. Update in database
    
    let mock_plant = PlantResponse {
        id,
        name: payload.name.unwrap_or_else(|| "Updated Plant".to_string()),
        genus: payload.genus.unwrap_or_else(|| "Updated Genus".to_string()),
        watering_interval_days: payload.watering_interval_days.unwrap_or(7),
        fertilizing_interval_days: payload.fertilizing_interval_days.unwrap_or(14),
        last_watered: None,
        last_fertilized: None,
        custom_metrics: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        user_id: Uuid::new_v4(),
    };

    tracing::info!("Updated plant: {}", mock_plant.name);
    Ok(Json(mock_plant))
}

async fn delete_plant(Path(id): Path<Uuid>) -> Result<StatusCode> {
    tracing::info!("Delete plant request for id: {}", id);

    // TODO: Implement actual plant deletion
    // In real implementation, would:
    // 1. Check if plant exists (return AppError::NotFound if not)
    // 2. Delete from database
    // 3. Handle any cascade deletions (photos, tracking entries, etc.)

    tracing::info!("Deleted plant with id: {}", id);
    Ok(StatusCode::NO_CONTENT)
}
