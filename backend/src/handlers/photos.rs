use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde_json::{json, Value};
use uuid::Uuid;

pub fn routes() -> Router {
    Router::new()
        .route(
            "/plants/:plant_id/photos",
            get(list_photos).post(upload_photo),
        )
        .route("/plants/:plant_id/photos/:photo_id", delete(delete_photo))
}

async fn list_photos(Path(plant_id): Path<Uuid>) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement actual photo listing
    tracing::info!("List photos request for plant: {}", plant_id);

    // Mock response for now
    let response = json!({
        "photos": [],
        "total": 0
    });

    Ok(Json(response))
}

async fn upload_photo(Path(plant_id): Path<Uuid>) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement actual photo upload
    tracing::info!("Upload photo request for plant: {}", plant_id);

    // Mock response for now
    let response = json!({
        "id": Uuid::new_v4(),
        "url": "https://example.com/photo.jpg",
        "thumbnail_url": "https://example.com/photo_thumb.jpg",
        "caption": null,
        "created_at": chrono::Utc::now(),
        "plant_id": plant_id
    });

    Ok(Json(response))
}

async fn delete_photo(
    Path((plant_id, photo_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    // TODO: Implement actual photo deletion
    tracing::info!(
        "Delete photo request for plant: {}, photo: {}",
        plant_id,
        photo_id
    );

    Ok(StatusCode::NO_CONTENT)
}
