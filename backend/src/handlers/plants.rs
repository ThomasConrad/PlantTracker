use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::models::{CreatePlantRequest, PlantResponse, PlantsResponse, UpdatePlantRequest};

pub fn routes() -> Router {
    Router::new()
        .route("/", get(list_plants).post(create_plant))
        .route("/:id", get(get_plant).put(update_plant).delete(delete_plant))
}

#[derive(Debug, Deserialize)]
struct ListPlantsQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<String>,
}

async fn list_plants(Query(params): Query<ListPlantsQuery>) -> Result<Json<PlantsResponse>, StatusCode> {
    // TODO: Implement actual plant listing
    tracing::info!("List plants request with params: {:?}", params);
    
    // Mock response for now
    let response = PlantsResponse {
        plants: vec![],
        total: 0,
        limit: params.limit.unwrap_or(20),
        offset: params.offset.unwrap_or(0),
    };
    
    Ok(Json(response))
}

async fn create_plant(Json(payload): Json<CreatePlantRequest>) -> Result<Json<PlantResponse>, StatusCode> {
    // TODO: Implement actual plant creation
    tracing::info!("Create plant request: {:?}", payload);
    
    // Mock response for now
    let mock_plant = PlantResponse {
        id: Uuid::new_v4(),
        name: payload.name,
        genus: payload.genus,
        watering_interval_days: payload.watering_interval_days,
        fertilizing_interval_days: payload.fertilizing_interval_days,
        last_watered: None,
        last_fertilized: None,
        custom_metrics: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        user_id: Uuid::new_v4(),
    };
    
    Ok(Json(mock_plant))
}

async fn get_plant(Path(id): Path<Uuid>) -> Result<Json<PlantResponse>, StatusCode> {
    // TODO: Implement actual plant retrieval
    tracing::info!("Get plant request for id: {}", id);
    
    // Mock response for now
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
    
    Ok(Json(mock_plant))
}

async fn update_plant(
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePlantRequest>,
) -> Result<Json<PlantResponse>, StatusCode> {
    // TODO: Implement actual plant update
    tracing::info!("Update plant request for id: {} with payload: {:?}", id, payload);
    
    // Mock response for now
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
    
    Ok(Json(mock_plant))
}

async fn delete_plant(Path(id): Path<Uuid>) -> Result<StatusCode, StatusCode> {
    // TODO: Implement actual plant deletion
    tracing::info!("Delete plant request for id: {}", id);
    
    Ok(StatusCode::NO_CONTENT)
}