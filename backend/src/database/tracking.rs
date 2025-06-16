use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use crate::database::DatabasePool;
use crate::models::tracking_entry::{
    CreateTrackingEntryRequest, EntryType, TrackingEntriesResponse, TrackingEntry,
};
use crate::utils::errors::AppError;

/// Get all tracking entries for a specific plant with pagination
pub async fn get_tracking_entries_for_plant_paginated(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
    limit: i64,
    offset: i64,
    sort_desc: bool,
    entry_type_filter: Option<&str>,
) -> Result<TrackingEntriesResponse, AppError> {
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

    // Build sort order
    let order_clause = if sort_desc {
        "ORDER BY timestamp DESC"
    } else {
        "ORDER BY timestamp ASC"
    };

    // Build filter clause for entry type
    let (filter_clause, count_filter_clause) = if let Some(_entry_type) = entry_type_filter {
        (" AND entry_type = ?", " AND entry_type = ?")
    } else {
        ("", "")
    };

    // Get total count
    let count_query = format!(
        "SELECT COUNT(*) as count FROM tracking_entries WHERE plant_id = ?{}",
        count_filter_clause
    );
    
    let total = if let Some(entry_type) = entry_type_filter {
        sqlx::query(&count_query)
            .bind(plant_id.to_string())
            .bind(entry_type)
            .fetch_one(pool)
            .await?
            .get::<i64, _>("count")
    } else {
        sqlx::query(&count_query)
            .bind(plant_id.to_string())
            .fetch_one(pool)
            .await?
            .get::<i64, _>("count")
    };

    // Get tracking entries with pagination
    let entries_query = format!(
        "SELECT id, plant_id, entry_type, timestamp, value, notes, metric_id, photo_ids, created_at, updated_at 
         FROM tracking_entries 
         WHERE plant_id = ?{} 
         {} 
         LIMIT ? OFFSET ?",
        filter_clause, order_clause
    );

    let entries_rows = if let Some(entry_type) = entry_type_filter {
        sqlx::query(&entries_query)
            .bind(plant_id.to_string())
            .bind(entry_type)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query(&entries_query)
            .bind(plant_id.to_string())
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
    };

    let entries: Vec<TrackingEntry> = entries_rows
        .into_iter()
        .map(|row| {
            let id_str: String = row.get("id");
            let plant_id_str: String = row.get("plant_id");
            let timestamp_str: String = row.get("timestamp");
            let created_at_str: String = row.get("created_at");
            let updated_at_str: String = row.get("updated_at");
            let entry_type_str: String = row.get("entry_type");
            let metric_id_str: Option<String> = row.get("metric_id");
            let value_str: Option<String> = row.get("value");
            let photo_ids_str: Option<String> = row.get("photo_ids");

            TrackingEntry {
                id: Uuid::parse_str(&id_str).expect("Invalid UUID"),
                plant_id: Uuid::parse_str(&plant_id_str).expect("Invalid UUID"),
                entry_type: match entry_type_str.as_str() {
                    "watering" => EntryType::Watering,
                    "fertilizing" => EntryType::Fertilizing,
                    "measurement" => EntryType::CustomMetric,
                    "note" => EntryType::Note,
                    "photo" => EntryType::Photo,
                    _ => EntryType::Watering, // fallback
                },
                timestamp: chrono::DateTime::parse_from_rfc3339(&timestamp_str)
                    .expect("Invalid timestamp")
                    .with_timezone(&Utc),
                value: value_str.and_then(|v| serde_json::from_str(&v).ok()),
                notes: row.get("notes"),
                metric_id: metric_id_str.and_then(|id| Uuid::parse_str(&id).ok()),
                photo_ids: photo_ids_str.and_then(|v| serde_json::from_str(&v).ok()),
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .expect("Invalid timestamp")
                    .with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                    .expect("Invalid timestamp")
                    .with_timezone(&Utc),
            }
        })
        .collect();

    Ok(TrackingEntriesResponse { entries, total })
}

/// Get all tracking entries for a specific plant
#[allow(dead_code)]
pub async fn get_tracking_entries_for_plant(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
) -> Result<TrackingEntriesResponse, AppError> {
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

    // Get tracking entries
    let entries_rows = sqlx::query(
        "SELECT id, plant_id, entry_type, timestamp, value, notes, metric_id, photo_ids, created_at, updated_at 
         FROM tracking_entries 
         WHERE plant_id = ? 
         ORDER BY timestamp DESC"
    )
    .bind(plant_id.to_string())
    .fetch_all(pool)
    .await?;

    let entries: Vec<TrackingEntry> = entries_rows
        .into_iter()
        .map(|row| {
            let id_str: String = row.get("id");
            let plant_id_str: String = row.get("plant_id");
            let timestamp_str: String = row.get("timestamp");
            let created_at_str: String = row.get("created_at");
            let updated_at_str: String = row.get("updated_at");
            let entry_type_str: String = row.get("entry_type");
            let metric_id_str: Option<String> = row.get("metric_id");
            let value_str: Option<String> = row.get("value");
            let photo_ids_str: Option<String> = row.get("photo_ids");

            TrackingEntry {
                id: Uuid::parse_str(&id_str).expect("Invalid UUID"),
                plant_id: Uuid::parse_str(&plant_id_str).expect("Invalid UUID"),
                entry_type: match entry_type_str.as_str() {
                    "watering" => EntryType::Watering,
                    "fertilizing" => EntryType::Fertilizing,
                    "measurement" => EntryType::CustomMetric,
                    "note" => EntryType::Note,
                    "photo" => EntryType::Photo,
                    _ => EntryType::Watering, // fallback
                },
                timestamp: chrono::DateTime::parse_from_rfc3339(&timestamp_str)
                    .expect("Invalid timestamp")
                    .with_timezone(&Utc),
                value: value_str.and_then(|v| serde_json::from_str(&v).ok()),
                notes: row.get("notes"),
                metric_id: metric_id_str.and_then(|id| Uuid::parse_str(&id).ok()),
                photo_ids: photo_ids_str.and_then(|v| serde_json::from_str(&v).ok()),
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .expect("Invalid timestamp")
                    .with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                    .expect("Invalid timestamp")
                    .with_timezone(&Utc),
            }
        })
        .collect();

    let total = entries.len() as i64;

    Ok(TrackingEntriesResponse { entries, total })
}

/// Create a new tracking entry for a plant
pub async fn create_tracking_entry(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
    request: &CreateTrackingEntryRequest,
) -> Result<TrackingEntry, AppError> {
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

    let entry_id = Uuid::new_v4();
    let now = Utc::now();

    let entry_type_str = match request.entry_type {
        EntryType::Watering => "watering",
        EntryType::Fertilizing => "fertilizing",
        EntryType::CustomMetric => "measurement",
        EntryType::Note => "note",
        EntryType::Photo => "photo",
    };

    let value_json = request
        .value
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_default());

    let photo_ids_json = request
        .photo_ids
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_default());

    // Create the tracking entry
    sqlx::query(
        "INSERT INTO tracking_entries (id, plant_id, entry_type, timestamp, value, notes, metric_id, photo_ids, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(entry_id.to_string())
    .bind(plant_id.to_string())
    .bind(entry_type_str)
    .bind(request.timestamp.to_rfc3339())
    .bind(&value_json)
    .bind(&request.notes)
    .bind(request.metric_id.map(|id| id.to_string()))
    .bind(&photo_ids_json)
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;

    // Update the plant's last watered/fertilized date if this is a watering or fertilizing entry
    match request.entry_type {
        EntryType::Watering => {
            sqlx::query(
                "UPDATE plants SET last_watered = ?, updated_at = ? WHERE id = ? AND user_id = ?",
            )
            .bind(request.timestamp.to_rfc3339())
            .bind(now.to_rfc3339())
            .bind(plant_id.to_string())
            .bind(user_id)
            .execute(pool)
            .await?;
        }
        EntryType::Fertilizing => {
            sqlx::query(
                "UPDATE plants SET last_fertilized = ?, updated_at = ? WHERE id = ? AND user_id = ?"
            )
            .bind(request.timestamp.to_rfc3339())
            .bind(now.to_rfc3339())
            .bind(plant_id.to_string())
            .bind(user_id)
            .execute(pool)
            .await?;
        }
        EntryType::CustomMetric => {
            // Custom metrics don't update plant care dates
        }
        EntryType::Note => {
            // Notes don't update plant care dates
        }
        EntryType::Photo => {
            // Photos don't update plant care dates
        }
    }

    Ok(TrackingEntry {
        id: entry_id,
        plant_id: *plant_id,
        entry_type: request.entry_type.clone(),
        timestamp: request.timestamp,
        value: request.value.clone(),
        notes: request.notes.clone(),
        metric_id: request.metric_id,
        photo_ids: request.photo_ids.as_ref().map(|v| serde_json::to_value(v).unwrap_or_default()),
        created_at: now,
        updated_at: now,
    })
}

/// Get a single tracking entry
pub async fn get_tracking_entry(
    pool: &DatabasePool,
    plant_id: &Uuid,
    entry_id: &Uuid,
    user_id: &str,
) -> Result<TrackingEntry, AppError> {
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

    // Get the specific tracking entry
    let entry_row = sqlx::query(
        "SELECT id, plant_id, entry_type, timestamp, value, notes, metric_id, photo_ids, created_at, updated_at 
         FROM tracking_entries 
         WHERE id = ? AND plant_id = ?"
    )
    .bind(entry_id.to_string())
    .bind(plant_id.to_string())
    .fetch_optional(pool)
    .await?;

    let row = entry_row.ok_or_else(|| AppError::NotFound {
        resource: format!("Tracking entry with id {entry_id}"),
    })?;

    let id_str: String = row.get("id");
    let plant_id_str: String = row.get("plant_id");
    let timestamp_str: String = row.get("timestamp");
    let created_at_str: String = row.get("created_at");
    let updated_at_str: String = row.get("updated_at");
    let entry_type_str: String = row.get("entry_type");
    let metric_id_str: Option<String> = row.get("metric_id");
    let value_str: Option<String> = row.get("value");
    let photo_ids_str: Option<String> = row.get("photo_ids");

    Ok(TrackingEntry {
        id: Uuid::parse_str(&id_str).expect("Invalid UUID"),
        plant_id: Uuid::parse_str(&plant_id_str).expect("Invalid UUID"),
        entry_type: match entry_type_str.as_str() {
            "watering" => EntryType::Watering,
            "fertilizing" => EntryType::Fertilizing,
            "measurement" => EntryType::CustomMetric,
            "note" => EntryType::Note,
            "photo" => EntryType::Photo,
            _ => EntryType::Watering, // fallback
        },
        timestamp: chrono::DateTime::parse_from_rfc3339(&timestamp_str)
            .expect("Invalid timestamp")
            .with_timezone(&Utc),
        value: value_str.and_then(|v| serde_json::from_str(&v).ok()),
        notes: row.get("notes"),
        metric_id: metric_id_str.and_then(|id| Uuid::parse_str(&id).ok()),
        photo_ids: photo_ids_str.and_then(|v| serde_json::from_str(&v).ok()),
        created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .expect("Invalid timestamp")
            .with_timezone(&Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .expect("Invalid timestamp")
            .with_timezone(&Utc),
    })
}

/// Update a tracking entry
pub async fn update_tracking_entry(
    pool: &DatabasePool,
    plant_id: &Uuid,
    entry_id: &Uuid,
    user_id: &str,
    request: &crate::models::tracking_entry::UpdateTrackingEntryRequest,
) -> Result<TrackingEntry, AppError> {
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

    // Verify the entry exists and belongs to this plant
    let entry_exists = sqlx::query("SELECT 1 FROM tracking_entries WHERE id = ? AND plant_id = ?")
        .bind(entry_id.to_string())
        .bind(plant_id.to_string())
        .fetch_optional(pool)
        .await?;

    if entry_exists.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Tracking entry with id {entry_id}"),
        });
    }

    let now = Utc::now();
    
    // Build dynamic update query based on provided fields
    let mut update_parts = vec!["updated_at = ?"];
    let mut values: Vec<String> = vec![now.to_rfc3339()];

    if let Some(timestamp) = &request.timestamp {
        update_parts.push("timestamp = ?");
        values.push(timestamp.to_rfc3339());
    }

    if let Some(value) = &request.value {
        update_parts.push("value = ?");
        values.push(serde_json::to_string(value).unwrap_or_default());
    }

    if let Some(notes) = &request.notes {
        update_parts.push("notes = ?");
        values.push(notes.clone());
    }

    if let Some(photo_ids) = &request.photo_ids {
        update_parts.push("photo_ids = ?");
        values.push(serde_json::to_string(photo_ids).unwrap_or_default());
    }

    let query = format!(
        "UPDATE tracking_entries SET {} WHERE id = ? AND plant_id = ?",
        update_parts.join(", ")
    );

    let mut query_builder = sqlx::query(&query);
    for value in values {
        query_builder = query_builder.bind(value);
    }
    query_builder = query_builder.bind(entry_id.to_string()).bind(plant_id.to_string());

    let result = query_builder.execute(pool).await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound {
            resource: format!("Tracking entry with id {entry_id}"),
        });
    }

    // Return the updated entry
    get_tracking_entry(pool, plant_id, entry_id, user_id).await
}

/// Delete a tracking entry
pub async fn delete_tracking_entry(
    pool: &DatabasePool,
    plant_id: &Uuid,
    entry_id: &Uuid,
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

    // Verify the entry exists and belongs to this plant
    let entry_row =
        sqlx::query("SELECT entry_type FROM tracking_entries WHERE id = ? AND plant_id = ?")
            .bind(entry_id.to_string())
            .bind(plant_id.to_string())
            .fetch_optional(pool)
            .await?;

    if entry_row.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Tracking entry with id {entry_id}"),
        });
    }

    // Delete the tracking entry
    let result = sqlx::query("DELETE FROM tracking_entries WHERE id = ? AND plant_id = ?")
        .bind(entry_id.to_string())
        .bind(plant_id.to_string())
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound {
            resource: format!("Tracking entry with id {entry_id}"),
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
    async fn test_get_tracking_entries_for_empty_plant() {
        let pool = setup_test_db().await;
        let (user_id, plant_id) = create_test_user_and_plant(&pool).await;

        let result = get_tracking_entries_for_plant(&pool, &plant_id, &user_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.entries.len(), 0);
        assert_eq!(response.total, 0);
    }

    #[tokio::test]
    async fn test_create_tracking_entry() {
        let pool = setup_test_db().await;
        let (user_id, plant_id) = create_test_user_and_plant(&pool).await;

        let request = CreateTrackingEntryRequest {
            entry_type: EntryType::Watering,
            timestamp: Utc::now(),
            value: None,
            notes: Some("Test watering".to_string()),
            metric_id: None,
            photo_ids: None,
        };

        let result = create_tracking_entry(&pool, &plant_id, &user_id, &request).await;
        assert!(result.is_ok());

        let entry = result.unwrap();
        assert_eq!(entry.plant_id, plant_id);
        assert!(matches!(entry.entry_type, EntryType::Watering));
        assert_eq!(entry.notes, Some("Test watering".to_string()));
    }

    #[tokio::test]
    async fn test_delete_tracking_entry() {
        let pool = setup_test_db().await;
        let (user_id, plant_id) = create_test_user_and_plant(&pool).await;

        // Create entry first
        let request = CreateTrackingEntryRequest {
            entry_type: EntryType::Fertilizing,
            timestamp: Utc::now(),
            value: None,
            notes: None,
            metric_id: None,
            photo_ids: None,
        };

        let entry = create_tracking_entry(&pool, &plant_id, &user_id, &request)
            .await
            .expect("Failed to create tracking entry");

        // Delete entry
        let result = delete_tracking_entry(&pool, &plant_id, &entry.id, &user_id).await;
        assert!(result.is_ok());

        // Verify entry is deleted
        let entries = get_tracking_entries_for_plant(&pool, &plant_id, &user_id)
            .await
            .expect("Failed to get tracking entries");
        assert_eq!(entries.entries.len(), 0);
    }

    #[tokio::test]
    async fn test_create_note_entry_with_photos() {
        let pool = setup_test_db().await;
        let (user_id, plant_id) = create_test_user_and_plant(&pool).await;

        let photo_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
        let request = CreateTrackingEntryRequest {
            entry_type: EntryType::Note,
            timestamp: Utc::now(),
            value: None,
            notes: Some("Growth observation with photos".to_string()),
            metric_id: None,
            photo_ids: Some(photo_ids.clone()),
        };

        let result = create_tracking_entry(&pool, &plant_id, &user_id, &request).await;
        if result.is_err() {
            eprintln!("Error creating note entry: {:?}", result);
        }
        assert!(result.is_ok());

        let entry = result.unwrap();
        assert_eq!(entry.plant_id, plant_id);
        assert!(matches!(entry.entry_type, EntryType::Note));
        assert_eq!(entry.notes, Some("Growth observation with photos".to_string()));
        
        // Verify photo_ids are stored correctly
        if let Some(stored_photo_ids) = entry.photo_ids {
            let parsed_ids: Vec<Uuid> = serde_json::from_value(stored_photo_ids).unwrap();
            assert_eq!(parsed_ids, photo_ids);
        } else {
            panic!("Photo IDs should be stored");
        }
    }

    #[tokio::test]
    async fn test_get_tracking_entry() {
        let pool = setup_test_db().await;
        let (user_id, plant_id) = create_test_user_and_plant(&pool).await;

        // Create an entry first
        let request = CreateTrackingEntryRequest {
            entry_type: EntryType::Fertilizing,
            timestamp: Utc::now(),
            value: None,
            notes: Some("Spring fertilizer".to_string()),
            metric_id: None,
            photo_ids: None,
        };

        let created_entry = create_tracking_entry(&pool, &plant_id, &user_id, &request)
            .await
            .expect("Failed to create tracking entry");

        // Retrieve the entry
        let result = get_tracking_entry(&pool, &plant_id, &created_entry.id, &user_id).await;
        assert!(result.is_ok());

        let retrieved_entry = result.unwrap();
        assert_eq!(retrieved_entry.id, created_entry.id);
        assert_eq!(retrieved_entry.plant_id, plant_id);
        assert!(matches!(retrieved_entry.entry_type, EntryType::Fertilizing));
        assert_eq!(retrieved_entry.notes, Some("Spring fertilizer".to_string()));
    }

    #[tokio::test]
    async fn test_get_tracking_entry_not_found() {
        let pool = setup_test_db().await;
        let (user_id, plant_id) = create_test_user_and_plant(&pool).await;

        let non_existent_id = Uuid::new_v4();
        let result = get_tracking_entry(&pool, &plant_id, &non_existent_id, &user_id).await;
        assert!(result.is_err());
        
        if let Err(AppError::NotFound { resource }) = result {
            assert!(resource.contains(&non_existent_id.to_string()));
        } else {
            panic!("Expected NotFound error");
        }
    }

    #[tokio::test]
    async fn test_update_tracking_entry() {
        let pool = setup_test_db().await;
        let (user_id, plant_id) = create_test_user_and_plant(&pool).await;

        // Create an entry first
        let request = CreateTrackingEntryRequest {
            entry_type: EntryType::Note,
            timestamp: Utc::now(),
            value: None,
            notes: Some("Initial note".to_string()),
            metric_id: None,
            photo_ids: None,
        };

        let created_entry = create_tracking_entry(&pool, &plant_id, &user_id, &request)
            .await
            .expect("Failed to create tracking entry");

        // Update the entry
        let new_timestamp = Utc::now();
        let photo_ids = vec![Uuid::new_v4()];
        let update_request = crate::models::tracking_entry::UpdateTrackingEntryRequest {
            timestamp: Some(new_timestamp),
            value: None,
            notes: Some("Updated note with more details".to_string()),
            photo_ids: Some(photo_ids.clone()),
        };

        let result = update_tracking_entry(&pool, &plant_id, &created_entry.id, &user_id, &update_request).await;
        assert!(result.is_ok());

        let updated_entry = result.unwrap();
        assert_eq!(updated_entry.id, created_entry.id);
        assert_eq!(updated_entry.notes, Some("Updated note with more details".to_string()));
        
        // Verify photo_ids are updated
        if let Some(stored_photo_ids) = updated_entry.photo_ids {
            let parsed_ids: Vec<Uuid> = serde_json::from_value(stored_photo_ids).unwrap();
            assert_eq!(parsed_ids, photo_ids);
        } else {
            panic!("Photo IDs should be updated");
        }
    }

    #[tokio::test]
    async fn test_update_tracking_entry_not_found() {
        let pool = setup_test_db().await;
        let (user_id, plant_id) = create_test_user_and_plant(&pool).await;

        let non_existent_id = Uuid::new_v4();
        let update_request = crate::models::tracking_entry::UpdateTrackingEntryRequest {
            timestamp: None,
            value: None,
            notes: Some("This should fail".to_string()),
            photo_ids: None,
        };

        let result = update_tracking_entry(&pool, &plant_id, &non_existent_id, &user_id, &update_request).await;
        assert!(result.is_err());
        
        if let Err(AppError::NotFound { resource }) = result {
            assert!(resource.contains(&non_existent_id.to_string()));
        } else {
            panic!("Expected NotFound error");
        }
    }

    #[tokio::test]
    async fn test_create_custom_metric_entry() {
        let pool = setup_test_db().await;
        let (user_id, plant_id) = create_test_user_and_plant(&pool).await;

        // First create a custom metric
        let metric_id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO custom_metrics (id, plant_id, name, unit, data_type, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(metric_id.to_string())
        .bind(plant_id.to_string())
        .bind("Height")
        .bind("cm")
        .bind("number")
        .bind(&now)
        .bind(&now)
        .execute(&pool)
        .await
        .expect("Failed to create custom metric");

        let request = CreateTrackingEntryRequest {
            entry_type: EntryType::CustomMetric,
            timestamp: Utc::now(),
            value: Some(serde_json::Value::Number(serde_json::Number::from(25))), // Height in cm
            notes: Some("Plant height measurement".to_string()),
            metric_id: Some(metric_id),
            photo_ids: None,
        };

        let result = create_tracking_entry(&pool, &plant_id, &user_id, &request).await;
        if result.is_err() {
            eprintln!("Error creating custom metric entry: {:?}", result);
        }
        assert!(result.is_ok());

        let entry = result.unwrap();
        assert_eq!(entry.plant_id, plant_id);
        assert!(matches!(entry.entry_type, EntryType::CustomMetric));
        assert_eq!(entry.metric_id, Some(metric_id));
        assert!(entry.value.is_some());
        
        if let Some(value) = entry.value {
            assert_eq!(value, serde_json::Value::Number(serde_json::Number::from(25)));
        }
    }

    #[tokio::test]
    async fn test_create_photo_entry() {
        let pool = setup_test_db().await;
        let (user_id, plant_id) = create_test_user_and_plant(&pool).await;

        let photo_ids = vec![Uuid::new_v4()];
        let request = CreateTrackingEntryRequest {
            entry_type: EntryType::Photo,
            timestamp: Utc::now(),
            value: None,
            notes: None,
            metric_id: None,
            photo_ids: Some(photo_ids.clone()),
        };

        let result = create_tracking_entry(&pool, &plant_id, &user_id, &request).await;
        if result.is_err() {
            eprintln!("Error creating photo entry: {:?}", result);
        }
        assert!(result.is_ok());

        let entry = result.unwrap();
        assert_eq!(entry.plant_id, plant_id);
        assert!(matches!(entry.entry_type, EntryType::Photo));
        
        // Verify photo_ids are stored correctly
        if let Some(stored_photo_ids) = entry.photo_ids {
            let parsed_ids: Vec<Uuid> = serde_json::from_value(stored_photo_ids).unwrap();
            assert_eq!(parsed_ids, photo_ids);
        } else {
            panic!("Photo IDs should be stored");
        }
    }

    #[tokio::test]
    async fn test_user_isolation_tracking_entries() {
        let pool = setup_test_db().await;
        let (user1_id, plant1_id) = create_test_user_and_plant(&pool).await;

        // Create second user and plant
        let user2_id = Uuid::new_v4().to_string();
        let plant2_id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO users (id, email, name, password_hash, salt, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&user2_id)
        .bind("test2@example.com")
        .bind("Test User 2")
        .bind("fake_hash")
        .bind("fake_salt")
        .bind(&now)
        .bind(&now)
        .execute(&pool)
        .await
        .expect("Failed to create second test user");

        sqlx::query(
            "INSERT INTO plants (id, user_id, name, genus, watering_interval_days, fertilizing_interval_days, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(plant2_id.to_string())
        .bind(&user2_id)
        .bind("Test Plant 2")
        .bind("Testus2")
        .bind(5)
        .bind(10)
        .bind(&now)
        .bind(&now)
        .execute(&pool)
        .await
        .expect("Failed to create second test plant");

        // Create entry for user 1
        let request1 = CreateTrackingEntryRequest {
            entry_type: EntryType::Watering,
            timestamp: Utc::now(),
            value: None,
            notes: Some("User 1 watering".to_string()),
            metric_id: None,
            photo_ids: None,
        };

        let entry1 = create_tracking_entry(&pool, &plant1_id, &user1_id, &request1)
            .await
            .expect("Failed to create entry for user 1");

        // User 2 should not be able to access user 1's entry
        let result = get_tracking_entry(&pool, &plant1_id, &entry1.id, &user2_id).await;
        assert!(result.is_err());
        
        // User 2 should not see user 1's entries when listing
        let entries_result = get_tracking_entries_for_plant(&pool, &plant1_id, &user2_id).await;
        assert!(entries_result.is_err());
    }
}
