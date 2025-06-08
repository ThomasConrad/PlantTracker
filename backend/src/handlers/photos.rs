use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::database::DatabasePool;
use crate::utils::errors::{AppError, Result};

pub fn routes() -> Router<DatabasePool> {
    Router::new()
        .route(
            "/plants/:plant_id/photos",
            get(list_photos).post(upload_photo),
        )
        .route("/plants/:plant_id/photos/:photo_id", delete(delete_photo))
}

async fn list_photos(Path(plant_id): Path<Uuid>) -> Result<Json<Value>> {
    tracing::info!("List photos request for plant: {}", plant_id);

    // TODO: Implement actual photo listing
    // In real implementation, would:
    // 1. Verify plant exists (return AppError::NotFound if not)
    // 2. Query photos from database
    
    // Mock response for now
    let response = json!({
        "photos": [],
        "total": 0
    });

    tracing::debug!("Returning 0 photos for plant: {}", plant_id);
    Ok(Json(response))
}

async fn upload_photo(Path(plant_id): Path<Uuid>) -> Result<Json<Value>> {
    tracing::info!("Upload photo request for plant: {}", plant_id);

    // TODO: Implement actual photo upload
    // In real implementation, would:
    // 1. Verify plant exists (return AppError::NotFound if not)
    // 2. Validate file upload (return AppError::Validation if invalid)
    // 3. Save file to storage
    // 4. Create thumbnail
    // 5. Save photo record to database
    
    let photo_id = Uuid::new_v4();
    let response = json!({
        "id": photo_id,
        "url": "https://example.com/photo.jpg",
        "thumbnail_url": "https://example.com/photo_thumb.jpg",
        "caption": null,
        "created_at": chrono::Utc::now(),
        "plant_id": plant_id
    });

    tracing::info!("Photo uploaded with id: {} for plant: {}", photo_id, plant_id);
    Ok(Json(response))
}

async fn delete_photo(
    Path((plant_id, photo_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
    tracing::info!(
        "Delete photo request for plant: {}, photo: {}",
        plant_id,
        photo_id
    );

    // TODO: Implement actual photo deletion
    // In real implementation, would:
    // 1. Verify plant exists (return AppError::NotFound if not)
    // 2. Verify photo exists and belongs to plant (return AppError::NotFound if not)
    // 3. Delete file from storage
    // 4. Delete photo record from database

    tracing::info!("Deleted photo: {} for plant: {}", photo_id, plant_id);
    Ok(StatusCode::NO_CONTENT)
}
