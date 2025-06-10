use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get},
    Router,
};
use uuid::Uuid;

use crate::auth::AuthSession;
use crate::database::{photos as db_photos, DatabasePool};
use crate::models::{PhotosResponse, UploadPhotoRequest};
use crate::utils::errors::{AppError, Result};

pub fn routes() -> Router<DatabasePool> {
    Router::new()
        .route(
            "/photos",
            get(list_photos).post(upload_photo),
        )
        .route("/photos/:photo_id", delete(delete_photo))
}

async fn list_photos(
    auth_session: AuthSession,
    State(pool): State<DatabasePool>,
    Path(plant_id): Path<Uuid>,
) -> Result<Json<PhotosResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!("List photos request for plant: {} by user: {}", plant_id, user.id);

    let response = db_photos::get_photos_for_plant(&pool, &plant_id, &user.id).await?;

    tracing::debug!("Returning {} photos for plant: {}", response.total, plant_id);
    Ok(Json(response))
}

async fn upload_photo(
    auth_session: AuthSession,
    State(pool): State<DatabasePool>,
    Path(plant_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<crate::models::Photo>)> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!("Upload photo request for plant: {} by user: {}", plant_id, user.id);

    let mut file_data: Option<Vec<u8>> = None;
    let mut original_filename: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut _caption: Option<String> = None;

    // Process multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|_e| AppError::Validation(
        validator::ValidationErrors::new()
    ))? {
        let name = field.name().unwrap_or("").to_string();
        
        match name.as_str() {
            "file" => {
                original_filename = field.file_name().map(|s| s.to_string());
                content_type = field.content_type().map(|s| s.to_string());
                file_data = Some(field.bytes().await.map_err(|_| AppError::Validation(
                    validator::ValidationErrors::new()
                ))?.to_vec());
            }
            "caption" => {
                _caption = Some(field.text().await.map_err(|_| AppError::Validation(
                    validator::ValidationErrors::new()
                ))?);
            }
            _ => {
                // Skip unknown fields
            }
        }
    }

    // Validate required fields
    let file_data = file_data.ok_or_else(|| AppError::Validation(
        validator::ValidationErrors::new()
    ))?;
    let original_filename = original_filename.ok_or_else(|| AppError::Validation(
        validator::ValidationErrors::new()
    ))?;
    let content_type = content_type.ok_or_else(|| AppError::Validation(
        validator::ValidationErrors::new()
    ))?;

    // Validate content type
    if !content_type.starts_with("image/") {
        return Err(AppError::Validation(validator::ValidationErrors::new()));
    }

    // Validate file size (10MB max)
    if file_data.len() > 10 * 1024 * 1024 {
        return Err(AppError::Validation(validator::ValidationErrors::new()));
    }

    // Create upload request
    let upload_request = UploadPhotoRequest {
        original_filename,
        size: file_data.len() as i64,
        content_type,
        data: file_data,
    };

    let photo = db_photos::create_photo(&pool, &plant_id, &user.id, &upload_request).await?;

    tracing::info!("Photo uploaded with id: {} for plant: {}", photo.id, plant_id);
    Ok((StatusCode::CREATED, Json(photo)))
}

async fn delete_photo(
    auth_session: AuthSession,
    State(pool): State<DatabasePool>,
    Path((plant_id, photo_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!(
        "Delete photo request for plant: {}, photo: {} by user: {}",
        plant_id,
        photo_id,
        user.id
    );

    db_photos::delete_photo(&pool, &plant_id, &photo_id, &user.id).await?;

    tracing::info!("Deleted photo: {} for plant: {}", photo_id, plant_id);
    Ok(StatusCode::NO_CONTENT)
}
