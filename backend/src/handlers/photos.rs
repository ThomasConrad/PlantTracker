use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{Json, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::AuthSession;
use crate::database::photos as db_photos;
use crate::models::{PhotoWithThumbnail, UploadPhotoRequest};
use crate::utils::errors::{AppError, Result};

#[derive(Debug, Deserialize)]
struct ListPhotosQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    sort: Option<String>, // "date_asc" or "date_desc" (default)
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PhotosResponse {
    photos: Vec<PhotoWithThumbnail>,
    total: i64,
    limit: i64,
    offset: i64,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/photos", get(list_photos).post(upload_photo))
        .route("/photos/:photo_id", get(serve_photo).delete(delete_photo))
        .route("/photos/:photo_id/thumbnail", get(serve_thumbnail))
}

async fn list_photos(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path(plant_id): Path<Uuid>,
    Query(params): Query<ListPhotosQuery>,
) -> Result<Json<PhotosResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!(
        "List photos request for plant: {} by user: {}",
        plant_id,
        user.id
    );

    // Parse query parameters
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    let sort_desc = match params.sort.as_deref() {
        Some("date_asc") => false,
        _ => true, // default to date_desc
    };

    let response = db_photos::get_photos_for_plant_paginated(
        &app_state.pool,
        &plant_id,
        &user.id,
        Some(limit),
        Some(offset),
        Some(sort_desc),
    )
    .await?;

    // Convert to PhotoWithThumbnail with URLs
    let photos_with_urls: Vec<PhotoWithThumbnail> = response
        .photos
        .into_iter()
        .map(|photo| {
            let full_url = format!(
                "/api/v1/plants/{}/photos/{}?v={}",
                plant_id,
                photo.id,
                photo.created_at.timestamp()
            );
            let thumbnail_url = if photo.thumbnail_width.is_some() {
                Some(format!(
                    "/api/v1/plants/{}/photos/{}/thumbnail?v={}",
                    plant_id,
                    photo.id,
                    photo.created_at.timestamp()
                ))
            } else {
                None
            };

            PhotoWithThumbnail {
                id: photo.id,
                plant_id: photo.plant_id,
                filename: photo.filename,
                original_filename: photo.original_filename,
                size: photo.size,
                content_type: photo.content_type,
                thumbnail_width: photo.thumbnail_width,
                thumbnail_height: photo.thumbnail_height,
                created_at: photo.created_at,
                full_url,
                thumbnail_url,
            }
        })
        .collect();

    tracing::debug!(
        "Returning {} of {} photos for plant: {}",
        photos_with_urls.len(),
        response.total,
        plant_id
    );

    Ok(Json(PhotosResponse {
        photos: photos_with_urls,
        total: response.total,
        limit,
        offset,
    }))
}

async fn serve_photo(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path((plant_id, photo_id)): Path<(Uuid, Uuid)>,
) -> Result<Response<Body>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!(
        "Serve photo request for plant: {}, photo: {} by user: {}",
        plant_id,
        photo_id,
        user.id
    );

    let (data, content_type) =
        db_photos::get_photo_data(&app_state.pool, &plant_id, &photo_id, &user.id).await?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, data.len())
        .header(header::CACHE_CONTROL, "public, max-age=31536000") // Cache for 1 year
        .header(header::ETAG, format!("\"{}-{}\"", plant_id, photo_id)) // ETag for caching
        .body(Body::from(data))
        .map_err(|_| AppError::Internal {
            message: "Failed to build response".to_string(),
        })?;

    tracing::debug!("Served photo: {} for plant: {}", photo_id, plant_id);
    Ok(response)
}

async fn upload_photo(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path(plant_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<crate::models::Photo>)> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!(
        "Upload photo request for plant: {} by user: {}",
        plant_id,
        user.id
    );

    let mut file_data: Option<Vec<u8>> = None;
    let mut original_filename: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut _caption: Option<String> = None;

    // Process multipart form data
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_e| AppError::Validation(validator::ValidationErrors::new()))?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
                original_filename = field.file_name().map(|s| s.to_string());
                content_type = field.content_type().map(|s| s.to_string());
                file_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| AppError::Validation(validator::ValidationErrors::new()))?
                        .to_vec(),
                );
            }
            "caption" => {
                _caption = Some(
                    field
                        .text()
                        .await
                        .map_err(|_| AppError::Validation(validator::ValidationErrors::new()))?,
                );
            }
            _ => {
                // Skip unknown fields
            }
        }
    }

    // Validate required fields
    let file_data =
        file_data.ok_or_else(|| AppError::Validation(validator::ValidationErrors::new()))?;
    let original_filename = original_filename
        .ok_or_else(|| AppError::Validation(validator::ValidationErrors::new()))?;
    let content_type =
        content_type.ok_or_else(|| AppError::Validation(validator::ValidationErrors::new()))?;

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
        generate_thumbnail: Some(true), // Always generate thumbnails
    };

    let photo = db_photos::create_photo(&app_state.pool, &plant_id, &user.id, &upload_request).await?;

    tracing::info!(
        "Photo uploaded with id: {} for plant: {}",
        photo.id,
        plant_id
    );
    Ok((StatusCode::CREATED, Json(photo)))
}

async fn delete_photo(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
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

    db_photos::delete_photo(&app_state.pool, &plant_id, &photo_id, &user.id).await?;

    tracing::info!("Deleted photo: {} for plant: {}", photo_id, plant_id);
    Ok(StatusCode::NO_CONTENT)
}

async fn serve_thumbnail(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path((plant_id, photo_id)): Path<(Uuid, Uuid)>,
) -> Result<Response<Body>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!(
        "Serve thumbnail request for plant: {}, photo: {} by user: {}",
        plant_id,
        photo_id,
        user.id
    );

    match db_photos::get_photo_thumbnail_data(&app_state.pool, &plant_id, &photo_id, &user.id).await {
        Ok((data, content_type)) => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CONTENT_LENGTH, data.len())
                .header(header::CACHE_CONTROL, "public, max-age=31536000") // Cache for 1 year
                .header(header::ETAG, format!("\"thumb-{}-{}\"", plant_id, photo_id)) // ETag for caching
                .body(Body::from(data))
                .map_err(|_| AppError::Internal {
                    message: "Failed to build response".to_string(),
                })?;

            tracing::debug!("Served thumbnail: {} for plant: {}", photo_id, plant_id);
            Ok(response)
        }
        Err(AppError::NotFound { .. }) => {
            // Thumbnail not ready yet, return 202 Accepted to indicate processing
            tracing::debug!(
                "Thumbnail not ready for photo: {} in plant: {}",
                photo_id,
                plant_id
            );
            let response = Response::builder()
                .status(StatusCode::ACCEPTED)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"status":"processing","message":"Thumbnail is being generated"}"#,
                ))
                .map_err(|_| AppError::Internal {
                    message: "Failed to build response".to_string(),
                })?;
            Ok(response)
        }
        Err(e) => Err(e),
    }
}
