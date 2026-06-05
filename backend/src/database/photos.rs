use chrono::Utc;
use uuid::Uuid;

use crate::database::DatabasePool;
use crate::models::{Photo, PhotosResponse, UploadPhotoRequest};
use crate::utils::db_traits::RequireAffected;
use crate::utils::errors::AppError;
use crate::utils::image_processing::process_uploaded_image;

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
    crate::utils::db_traits::verify_plant_ownership(pool, plant_id, user_id).await?;

    let plant_id_str = plant_id.to_string();

    // Set default values
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    let sort_desc = sort_desc.unwrap_or(true);

    // Get total count
    let total_row = sqlx::query!(
        r#"SELECT COUNT(*) as count FROM photos WHERE plant_id = ?"#,
        plant_id_str
    )
    .fetch_one(pool)
    .await?;
    let total: i64 = total_row.count.into();

    // Build sort order - dynamic query, cannot use macro
    let order_clause = if sort_desc {
        "ORDER BY created_at DESC"
    } else {
        "ORDER BY created_at ASC"
    };

    // Get photos (without data to save memory for listings) with pagination
    let query = format!(
        "SELECT id, plant_id, filename, original_filename, size, content_type, width, height, created_at 
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
            use sqlx::Row;
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
                width: row.get("width"),
                height: row.get("height"),
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
    crate::utils::db_traits::verify_plant_ownership(pool, plant_id, user_id).await?;

    // Get photo data
    let plant_id_str = plant_id.to_string();
    let photo_id_str = photo_id.to_string();
    let photo_row = sqlx::query!(
        r#"SELECT data, content_type FROM photos WHERE id = ? AND plant_id = ?"#,
        photo_id_str,
        plant_id_str
    )
    .fetch_optional(pool)
    .await?;

    match photo_row {
        Some(row) => {
            let data: Vec<u8> = row.data;
            let content_type: String = row.content_type;
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
    crate::utils::db_traits::verify_plant_ownership(pool, plant_id, user_id).await?;

    let plant_id_str = plant_id.to_string();

    let photo_id = Uuid::new_v4();
    let now = Utc::now();

    // Process the uploaded image to AVIF with 4K cropping
    let processed_image = process_uploaded_image(&request.data, &request.content_type)
        .await
        .map_err(|e| {
            tracing::error!("Failed to process uploaded image: {:?}", e);
            AppError::Validation(validator::ValidationErrors::new())
        })?;

    // Generate unique filename with AVIF extension
    let filename = format!("{}_{}.avif", plant_id, photo_id);

    // Store processed AVIF image data in database
    let photo_id_str = photo_id.to_string();
    let size = processed_image.data.len() as i64;
    let width = processed_image.width as i32;
    let height = processed_image.height as i32;
    let created_at_str = now.to_rfc3339();
    sqlx::query!(
        r#"INSERT INTO photos (id, plant_id, filename, original_filename, size, content_type, data, width, height, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        photo_id_str,
        plant_id_str,
        filename,
        request.original_filename,
        size,
        processed_image.content_type,
        processed_image.data,
        width,
        height,
        created_at_str
    )
    .execute(pool)
    .await?;

    tracing::info!(
        "Successfully processed and stored image: {} bytes -> {} bytes AVIF ({}x{})",
        request.data.len(),
        processed_image.data.len(),
        processed_image.width,
        processed_image.height
    );

    Ok(Photo {
        id: photo_id,
        plant_id: *plant_id,
        filename,
        original_filename: request.original_filename.clone(),
        size: processed_image.data.len() as i64,
        content_type: processed_image.content_type,
        width: Some(processed_image.width as i32),
        height: Some(processed_image.height as i32),
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
    crate::utils::db_traits::verify_plant_ownership(pool, plant_id, user_id).await?;

    let plant_id_str = plant_id.to_string();

    // Verify photo exists before deletion
    let photo_id_str = photo_id.to_string();
    let photo_row = sqlx::query!(
        r#"SELECT 1 as exists_flag FROM photos WHERE id = ? AND plant_id = ?"#,
        photo_id_str,
        plant_id_str
    )
    .fetch_optional(pool)
    .await?;

    if photo_row.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Photo with id {photo_id}"),
        });
    }

    let mut tx = pool.begin().await?;

    // Remove this photo reference from historical tracking entries so old activity rows do not
    // point to now-missing images.
    let entries_with_photos = sqlx::query!(
        r#"SELECT id, photo_ids FROM tracking_entries WHERE plant_id = ? AND photo_ids IS NOT NULL"#,
        plant_id_str
    )
    .fetch_all(&mut *tx)
    .await?;

    let deleted_photo_id = photo_id.to_string();
    for row in entries_with_photos {
        let entry_id: String = row.id;
        let photo_ids_raw: Option<String> = row.photo_ids;

        let Some(photo_ids_raw) = photo_ids_raw else {
            continue;
        };

        let Ok(mut photo_ids) = serde_json::from_str::<Vec<String>>(&photo_ids_raw) else {
            tracing::warn!(
                "Skipping malformed photo_ids JSON for tracking entry {}",
                entry_id
            );
            continue;
        };

        let original_len = photo_ids.len();
        photo_ids.retain(|id| id != &deleted_photo_id);

        if photo_ids.len() != original_len {
            let updated_photo_ids: Option<String> = if photo_ids.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&photo_ids).unwrap_or_default())
            };

            let updated_at = Utc::now().to_rfc3339();
            sqlx::query!(
                r#"UPDATE tracking_entries SET photo_ids = ?, updated_at = ? WHERE id = ? AND plant_id = ?"#,
                updated_photo_ids,
                updated_at,
                entry_id,
                plant_id_str
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    // Delete photo record (photo data is deleted with the row)
    sqlx::query!(
        r#"DELETE FROM photos WHERE id = ? AND plant_id = ?"#,
        photo_id_str,
        plant_id_str
    )
    .execute(&mut *tx)
    .await?
    .require_affected(format!("Photo with id {photo_id}"))?;

    tx.commit().await?;

    Ok(())
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
        let plant_id_str = plant_id.to_string();

        // Create user
        let email = "test@example.com";
        let name = "Test User";
        let password_hash = "fake_hash";
        let role = "user";
        let can_create_invites = false;
        let max_invites: i32 = 5;
        let invites_created: i32 = 0;
        sqlx::query!(
            r#"INSERT INTO users (id, email, name, password_hash, role, can_create_invites, max_invites, invites_created, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            user_id,
            email,
            name,
            password_hash,
            role,
            can_create_invites,
            max_invites,
            invites_created,
            now,
            now
        )
        .execute(pool)
        .await
        .expect("Failed to create test user");

        // Create plant
        let plant_name = "Test Plant";
        let genus = "Testus";
        sqlx::query!(
            r#"INSERT INTO plants (id, user_id, name, genus, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)"#,
            plant_id_str,
            user_id,
            plant_name,
            genus,
            now,
            now
        )
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

        // Create a valid 1x1 pixel JPEG using the image crate
        use image::{DynamicImage, ImageOutputFormat};
        use std::io::Cursor;

        let img = DynamicImage::new_rgb8(1, 1);
        let mut jpeg_data = Vec::new();
        img.write_to(
            &mut Cursor::new(&mut jpeg_data),
            ImageOutputFormat::Jpeg(80),
        )
        .unwrap();

        let request = UploadPhotoRequest {
            original_filename: "test.jpg".to_string(),
            size: jpeg_data.len() as i64,
            content_type: "image/jpeg".to_string(),
            data: jpeg_data,
        };

        let result = create_photo(&pool, &plant_id, &user_id, &request).await;
        assert!(result.is_ok());

        let photo = result.unwrap();
        assert_eq!(photo.plant_id, plant_id);
        assert_eq!(photo.original_filename, "test.jpg");
        assert_eq!(photo.content_type, "image/webp"); // Should be converted to WebP
        assert!(photo.size > 0); // Size will be different after WebP conversion
        assert!(photo.width.is_some());
        assert!(photo.height.is_some());
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
        };

        let result = create_photo(&pool, &plant_id, &user_id, &request).await;
        assert!(matches!(result, Err(AppError::NotFound { .. })));
    }

    #[tokio::test]
    async fn test_delete_photo() {
        let pool = setup_test_db().await;
        let (user_id, plant_id) = create_test_user_and_plant(&pool).await;

        // Create a valid JPEG image
        use image::{DynamicImage, ImageOutputFormat};
        use std::io::Cursor;

        let img = DynamicImage::new_rgb8(5, 5);
        let mut jpeg_data = Vec::new();
        img.write_to(
            &mut Cursor::new(&mut jpeg_data),
            ImageOutputFormat::Jpeg(80),
        )
        .unwrap();

        // Create photo first
        let request = UploadPhotoRequest {
            original_filename: "test.jpg".to_string(),
            size: jpeg_data.len() as i64,
            content_type: "image/jpeg".to_string(),
            data: jpeg_data,
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

        // Create a valid JPEG image
        use image::{DynamicImage, ImageOutputFormat};
        use std::io::Cursor;

        let img = DynamicImage::new_rgb8(10, 10);
        let mut jpeg_data = Vec::new();
        img.write_to(
            &mut Cursor::new(&mut jpeg_data),
            ImageOutputFormat::Jpeg(80),
        )
        .unwrap();

        // Create photo first
        let request = UploadPhotoRequest {
            original_filename: "test.jpg".to_string(),
            size: jpeg_data.len() as i64,
            content_type: "image/jpeg".to_string(),
            data: jpeg_data,
        };

        let photo = create_photo(&pool, &plant_id, &user_id, &request)
            .await
            .expect("Failed to create photo");

        // Get photo data
        let result = get_photo_data(&pool, &plant_id, &photo.id, &user_id).await;
        assert!(result.is_ok());

        let (data, content_type) = result.unwrap();
        // Data will be different after WebP conversion
        assert!(!data.is_empty());
        assert_eq!(content_type, "image/webp");
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
