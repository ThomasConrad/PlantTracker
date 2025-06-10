use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use crate::database::DatabasePool;
use crate::models::tracking_entry::{
    CreateTrackingEntryRequest, EntryType, TrackingEntriesResponse, TrackingEntry,
};
use crate::utils::errors::AppError;

/// Get all tracking entries for a specific plant
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
        "SELECT id, plant_id, entry_type, timestamp, value, notes, metric_id, created_at, updated_at 
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

            TrackingEntry {
                id: Uuid::parse_str(&id_str).expect("Invalid UUID"),
                plant_id: Uuid::parse_str(&plant_id_str).expect("Invalid UUID"),
                entry_type: match entry_type_str.as_str() {
                    "watering" => EntryType::Watering,
                    "fertilizing" => EntryType::Fertilizing,
                    "measurement" => EntryType::CustomMetric,
                    _ => EntryType::Watering, // fallback
                },
                timestamp: chrono::DateTime::parse_from_rfc3339(&timestamp_str)
                    .expect("Invalid timestamp")
                    .with_timezone(&Utc),
                value: value_str.and_then(|v| serde_json::from_str(&v).ok()),
                notes: row.get("notes"),
                metric_id: metric_id_str.and_then(|id| Uuid::parse_str(&id).ok()),
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
    };

    let value_json = request
        .value
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_default());

    // Create the tracking entry
    sqlx::query(
        "INSERT INTO tracking_entries (id, plant_id, entry_type, timestamp, value, notes, metric_id, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(entry_id.to_string())
    .bind(plant_id.to_string())
    .bind(entry_type_str)
    .bind(request.timestamp.to_rfc3339())
    .bind(&value_json)
    .bind(&request.notes)
    .bind(request.metric_id.map(|id| id.to_string()))
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
    }

    Ok(TrackingEntry {
        id: entry_id,
        plant_id: *plant_id,
        entry_type: request.entry_type.clone(),
        timestamp: request.timestamp,
        value: request.value.clone(),
        notes: request.notes.clone(),
        metric_id: request.metric_id,
        created_at: now,
        updated_at: now,
    })
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
}
