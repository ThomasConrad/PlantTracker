use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use crate::database::DatabasePool;
use crate::models::{Photo, PhotosResponse, UploadPhotoRequest};
use crate::utils::errors::AppError;

/// Get all photos for a specific plant
pub async fn get_photos_for_plant(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
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

    // Get photos (without data to save memory for listings)
    let photos_rows = sqlx::query(
        "SELECT id, plant_id, filename, original_filename, size, content_type, created_at 
         FROM photos 
         WHERE plant_id = ? 
         ORDER BY created_at DESC",
    )
    .bind(plant_id.to_string())
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
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .expect("Invalid timestamp")
                    .with_timezone(&Utc),
            }
        })
        .collect();

    let total = photos.len() as i64;

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

    // Store photo data in database
    sqlx::query(
        "INSERT INTO photos (id, plant_id, filename, original_filename, size, content_type, data, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(photo_id.to_string())
    .bind(plant_id.to_string())
    .bind(&filename)
    .bind(&request.original_filename)
    .bind(request.size)
    .bind(&request.content_type)
    .bind(&request.data)
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(Photo {
        id: photo_id,
        plant_id: *plant_id,
        filename,
        original_filename: request.original_filename.clone(),
        size: request.size,
        content_type: request.content_type.clone(),
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
