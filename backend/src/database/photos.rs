use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use crate::database::DatabasePool;
use crate::models::{Photo, PhotosResponse, UploadPhotoRequest};
use crate::utils::errors::AppError;
use crate::utils::thumbnail::generate_thumbnail;

/// Get all photos for a specific plant
#[allow(dead_code)]
pub async fn get_photos_for_plant(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
) -> Result<PhotosResponse, AppError> {
    get_photos_for_plant_paginated(pool, plant_id, user_id, None, None, None).await
}

/// Get photos for a specific plant with pagination
pub async fn get_photos_for_plant_paginated(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
    limit: Option<i64>,
    offset: Option<i64>,
    sort_desc: Option<bool>,
) -> Result<PhotosResponse, AppError> {
    // First verify the plant exists and belongs to the user
    let plant_exists = sqlx::query("SELECT 1 FROM plants WHERE id = ? AND user_id = ?")
        .bind(plant_id.to_string())
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    if plant_exists.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    // Set default values
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    let sort_desc = sort_desc.unwrap_or(true);
    
    // Get total count
    let total_row = sqlx::query("SELECT COUNT(*) as count FROM photos WHERE plant_id = ?")
        .bind(plant_id.to_string())
        .fetch_one(pool)
        .await?;
    let total: i64 = total_row.get("count");
    
    // Build sort order
    let order_clause = if sort_desc {
        "ORDER BY created_at DESC"
    } else {
        "ORDER BY created_at ASC"
    };
    
    // Get photos (without data to save memory for listings) with pagination
    let query = format!(
        "SELECT id, plant_id, filename, original_filename, size, content_type, thumbnail_width, thumbnail_height, created_at 
         FROM photos 
         WHERE plant_id = ? 
         {} 
         LIMIT ? OFFSET ?",
        order_clause
    );
    
    let photos_rows = sqlx::query(&query)
        .bind(plant_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let photos: Vec<Photo> = photos_rows
        .into_iter()
        .map(|row| {
            let id_str: String = row.get("id");
            let plant_id_str: String = row.get("plant_id");
            let created_at_str: String = row.get("created_at");

            Photo {
                id: Uuid::parse_str(&id_str).expect("Invalid UUID"),
                plant_id: Uuid::parse_str(&plant_id_str).expect("Invalid UUID"),
                filename: row.get("filename"),
                original_filename: row.get("original_filename"),
                size: row.get("size"),
                content_type: row.get("content_type"),
                thumbnail_width: row.get("thumbnail_width"),
                thumbnail_height: row.get("thumbnail_height"),
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .expect("Invalid timestamp")
                    .with_timezone(&Utc),
            }
        })
        .collect();

    Ok(PhotosResponse { photos, total })
}

/// Get a single photo with its data for serving
pub async fn get_photo_data(
    pool: &DatabasePool,
    plant_id: &Uuid,
    photo_id: &Uuid,
    user_id: &str,
) -> Result<(Vec<u8>, String), AppError> {
    // First verify the plant exists and belongs to the user
    let plant_exists = sqlx::query("SELECT 1 FROM plants WHERE id = ? AND user_id = ?")
        .bind(plant_id.to_string())
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    if plant_exists.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    // Get photo data
    let photo_row =
        sqlx::query("SELECT data, content_type FROM photos WHERE id = ? AND plant_id = ?")
            .bind(photo_id.to_string())
            .bind(plant_id.to_string())
            .fetch_optional(pool)
            .await?;

    match photo_row {
        Some(row) => {
            let data: Vec<u8> = row.get("data");
            let content_type: String = row.get("content_type");
            Ok((data, content_type))
        }
        None => Err(AppError::NotFound {
            resource: format!("Photo with id {photo_id}"),
        }),
    }
}

/// Upload a new photo for a plant
pub async fn create_photo(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
    request: &UploadPhotoRequest,
) -> Result<Photo, AppError> {
    // First verify the plant exists and belongs to the user
    let plant_exists = sqlx::query("SELECT 1 FROM plants WHERE id = ? AND user_id = ?")
        .bind(plant_id.to_string())
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    if plant_exists.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    let photo_id = Uuid::new_v4();
    let now = Utc::now();

    // Generate unique filename
    let extension = match request.content_type.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => return Err(AppError::Validation(validator::ValidationErrors::new())),
    };
    let filename = format!("{}_{}.{}", plant_id, photo_id, extension);

    // Store photo data in database first (without thumbnail)
    sqlx::query(
        "INSERT INTO photos (id, plant_id, filename, original_filename, size, content_type, data, thumbnail_data, thumbnail_width, thumbnail_height, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(photo_id.to_string())
    .bind(plant_id.to_string())
    .bind(&filename)
    .bind(&request.original_filename)
    .bind(request.size)
    .bind(&request.content_type)
    .bind(&request.data)
    .bind(None::<Vec<u8>>) // thumbnail_data - will be updated later
    .bind(None::<i32>) // thumbnail_width - will be updated later  
    .bind(None::<i32>) // thumbnail_height - will be updated later
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;

    // Generate thumbnail asynchronously if requested
    if request.generate_thumbnail.unwrap_or(true) {
        let pool_clone = pool.clone();
        let photo_id_clone = photo_id;
        let data_clone = request.data.clone();
        let content_type_clone = request.content_type.clone();
        
        tokio::spawn(async move {
            let thumbnail_start = std::time::Instant::now();
            match generate_thumbnail(&data_clone, &content_type_clone) {
                Ok(thumbnail_info) => {
                    let duration = thumbnail_start.elapsed();
                    tracing::info!(
                        "Async thumbnail generation took {:.2}ms for {} bytes ({:.1}MB)",
                        duration.as_millis(),
                        data_clone.len(),
                        data_clone.len() as f64 / (1024.0 * 1024.0)
                    );
                    
                    // Update the photo record with thumbnail data
                    if let Err(e) = sqlx::query(
                        "UPDATE photos SET thumbnail_data = ?, thumbnail_width = ?, thumbnail_height = ? WHERE id = ?"
                    )
                    .bind(&thumbnail_info.data)
                    .bind(thumbnail_info.width)
                    .bind(thumbnail_info.height)
                    .bind(photo_id_clone.to_string())
                    .execute(&pool_clone)
                    .await {
                        tracing::error!("Failed to update photo with thumbnail data: {:?}", e);
                    } else {
                        tracing::debug!("Successfully updated photo {} with thumbnail", photo_id_clone);
                    }
                },
                Err(e) => {
                    let duration = thumbnail_start.elapsed();
                    tracing::warn!(
                        "Async thumbnail generation failed after {:.2}ms for photo {}: {:?}", 
                        duration.as_millis(), 
                        photo_id_clone, 
                        e
                    );
                }
            }
        });
    }

    Ok(Photo {
        id: photo_id,
        plant_id: *plant_id,
        filename,
        original_filename: request.original_filename.clone(),
        size: request.size,
        content_type: request.content_type.clone(),
        thumbnail_width: None, // Will be updated asynchronously
        thumbnail_height: None, // Will be updated asynchronously
        created_at: now,
    })
}

/// Delete a photo
pub async fn delete_photo(
    pool: &DatabasePool,
    plant_id: &Uuid,
    photo_id: &Uuid,
    user_id: &str,
) -> Result<(), AppError> {
    // First verify the plant exists and belongs to the user
    let plant_exists = sqlx::query("SELECT 1 FROM plants WHERE id = ? AND user_id = ?")
        .bind(plant_id.to_string())
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    if plant_exists.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    // Verify photo exists before deletion
    let photo_row = sqlx::query("SELECT 1 FROM photos WHERE id = ? AND plant_id = ?")
        .bind(photo_id.to_string())
        .bind(plant_id.to_string())
        .fetch_optional(pool)
        .await?;

    if photo_row.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Photo with id {photo_id}"),
        });
    }

    // Photo data will be automatically deleted with the record

    // Delete photo record
    let result = sqlx::query("DELETE FROM photos WHERE id = ? AND plant_id = ?")
        .bind(photo_id.to_string())
        .bind(plant_id.to_string())
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound {
            resource: format!("Photo with id {photo_id}"),
        });
    }

    Ok(())
}

/// Get thumbnail data for a photo
pub async fn get_photo_thumbnail_data(
    pool: &DatabasePool,
    plant_id: &Uuid,
    photo_id: &Uuid,
    user_id: &str,
) -> Result<(Vec<u8>, String), AppError> {
    // First verify the plant exists and belongs to the user
    let plant_exists = sqlx::query("SELECT 1 FROM plants WHERE id = ? AND user_id = ?")
        .bind(plant_id.to_string())
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    if plant_exists.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    // Get thumbnail data
    let photo_row = sqlx::query(
        "SELECT thumbnail_data, content_type FROM photos WHERE id = ? AND plant_id = ? AND thumbnail_data IS NOT NULL"
    )
    .bind(photo_id.to_string())
    .bind(plant_id.to_string())
    .fetch_optional(pool)
    .await?;

    match photo_row {
        Some(row) => {
            let data: Vec<u8> = row.get("thumbnail_data");
            let _content_type: String = row.get("content_type");
            // Thumbnails are always JPEG
            Ok((data, "image/jpeg".to_string()))
        }
        None => Err(AppError::NotFound {
            resource: format!("Thumbnail for photo with id {photo_id}"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::create_pool_with_url;

    async fn setup_test_db() -> DatabasePool {
        let pool = create_pool_with_url("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        crate::database::run_migrations(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    async fn create_test_user_and_plant(pool: &DatabasePool) -> (String, Uuid) {
        let user_id = Uuid::new_v4().to_string();
        let plant_id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();

        // Create user
        sqlx::query(
            "INSERT INTO users (id, email, name, password_hash, salt, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&user_id)
        .bind("test@example.com")
        .bind("Test User")
        .bind("fake_hash")
        .bind("fake_salt")
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await
        .expect("Failed to create test user");

        // Create plant
        sqlx::query(
            "INSERT INTO plants (id, user_id, name, genus, watering_interval_days, fertilizing_interval_days, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(plant_id.to_string())
        .bind(&user_id)
        .bind("Test Plant")
        .bind("Testus")
        .bind(7)
        .bind(14)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await
        .expect("Failed to create test plant");

        (user_id, plant_id)
    }

    #[tokio::test]
    async fn test_get_photos_for_nonexistent_plant() {
        let pool = setup_test_db().await;
        let user_id = Uuid::new_v4().to_string();
        let plant_id = Uuid::new_v4();

        let result = get_photos_for_plant(&pool, &plant_id, &user_id).await;
        assert!(matches!(result, Err(AppError::NotFound { .. })));
    }

    #[tokio::test]
    async fn test_get_photos_for_empty_plant() {
        let pool = setup_test_db().await;
        let (user_id, plant_id) = create_test_user_and_plant(&pool).await;

        let result = get_photos_for_plant(&pool, &plant_id, &user_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.photos.len(), 0);
        assert_eq!(response.total, 0);
    }

    #[tokio::test]
    async fn test_create_photo() {
        let pool = setup_test_db().await;
        let (user_id, plant_id) = create_test_user_and_plant(&pool).await;

        let request = UploadPhotoRequest {
            original_filename: "test.jpg".to_string(),
            size: 1024,
            content_type: "image/jpeg".to_string(),
            data: vec![1, 2, 3, 4], // Fake image data
            generate_thumbnail: Some(false), // Skip thumbnail generation in tests
        };

        let result = create_photo(&pool, &plant_id, &user_id, &request).await;
        assert!(result.is_ok());

        let photo = result.unwrap();
        assert_eq!(photo.plant_id, plant_id);
        assert_eq!(photo.original_filename, "test.jpg");
        assert_eq!(photo.size, 1024);
        assert_eq!(photo.content_type, "image/jpeg");
        assert!(photo.filename.contains(&plant_id.to_string()));
    }

    #[tokio::test]
    async fn test_create_photo_for_nonexistent_plant() {
        let pool = setup_test_db().await;
        let user_id = Uuid::new_v4().to_string();
        let plant_id = Uuid::new_v4();

        let request = UploadPhotoRequest {
            original_filename: "test.jpg".to_string(),
            size: 1024,
            content_type: "image/jpeg".to_string(),
            data: vec![1, 2, 3, 4],
            generate_thumbnail: Some(false),
        };

        let result = create_photo(&pool, &plant_id, &user_id, &request).await;
        assert!(matches!(result, Err(AppError::NotFound { .. })));
    }

    #[tokio::test]
    async fn test_delete_photo() {
        let pool = setup_test_db().await;
        let (user_id, plant_id) = create_test_user_and_plant(&pool).await;

        // Create photo first
        let request = UploadPhotoRequest {
            original_filename: "test.jpg".to_string(),
            size: 1024,
            content_type: "image/jpeg".to_string(),
            data: vec![1, 2, 3, 4],
            generate_thumbnail: Some(false),
        };

        let photo = create_photo(&pool, &plant_id, &user_id, &request)
            .await
            .expect("Failed to create photo");

        // Delete photo
        let result = delete_photo(&pool, &plant_id, &photo.id, &user_id).await;
        assert!(result.is_ok());

        // Verify photo is deleted
        let photos = get_photos_for_plant(&pool, &plant_id, &user_id)
            .await
            .expect("Failed to get photos");
        assert_eq!(photos.photos.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_nonexistent_photo() {
        let pool = setup_test_db().await;
        let (user_id, plant_id) = create_test_user_and_plant(&pool).await;
        let photo_id = Uuid::new_v4();

        let result = delete_photo(&pool, &plant_id, &photo_id, &user_id).await;
        assert!(matches!(result, Err(AppError::NotFound { .. })));
    }

    #[tokio::test]
    async fn test_get_photo_data() {
        let pool = setup_test_db().await;
        let (user_id, plant_id) = create_test_user_and_plant(&pool).await;

        let fake_image_data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header

        // Create photo first
        let request = UploadPhotoRequest {
            original_filename: "test.jpg".to_string(),
            size: fake_image_data.len() as i64,
            content_type: "image/jpeg".to_string(),
            data: fake_image_data.clone(),
            generate_thumbnail: Some(false),
        };

        let photo = create_photo(&pool, &plant_id, &user_id, &request)
            .await
            .expect("Failed to create photo");

        // Get photo data
        let result = get_photo_data(&pool, &plant_id, &photo.id, &user_id).await;
        assert!(result.is_ok());

        let (data, content_type) = result.unwrap();
        assert_eq!(data, fake_image_data);
        assert_eq!(content_type, "image/jpeg");
    }

    #[tokio::test]
    async fn test_get_photo_data_for_nonexistent_photo() {
        let pool = setup_test_db().await;
        let (user_id, plant_id) = create_test_user_and_plant(&pool).await;
        let photo_id = Uuid::new_v4();

        let result = get_photo_data(&pool, &plant_id, &photo_id, &user_id).await;
        assert!(matches!(result, Err(AppError::NotFound { .. })));
    }
}
