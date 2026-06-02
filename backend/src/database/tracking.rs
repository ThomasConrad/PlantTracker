use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use crate::database::DatabasePool;
use crate::models::tracking_entry::{
    CreateTrackingEntryRequest, Measurement, TrackingEntriesResponse, TrackingEntry,
};
use crate::utils::errors::AppError;

fn row_to_entry(row: &sqlx::sqlite::SqliteRow) -> TrackingEntry {
    let id_str: String = row.get("id");
    let plant_id_str: String = row.get("plant_id");
    let timestamp_str: String = row.get("timestamp");
    let created_at_str: String = row.get("created_at");
    let updated_at_str: String = row.get("updated_at");
    let care_task_ids_str: Option<String> = row.get("care_task_ids");
    let measurements_str: Option<String> = row.get("measurements");
    let photo_ids_str: Option<String> = row.get("photo_ids");

    let care_task_ids: Option<Vec<Uuid>> = care_task_ids_str.and_then(|s| {
        serde_json::from_str::<Vec<String>>(&s)
            .ok()
            .map(|ids| ids.iter().filter_map(|id| Uuid::parse_str(id).ok()).collect())
    });

    let measurements: Option<Vec<Measurement>> =
        measurements_str.and_then(|s| serde_json::from_str(&s).ok());

    let photo_ids: Option<Vec<Uuid>> = photo_ids_str.and_then(|s| {
        serde_json::from_str::<Vec<String>>(&s)
            .ok()
            .map(|ids| ids.iter().filter_map(|id| Uuid::parse_str(id).ok()).collect())
    });

    TrackingEntry {
        id: Uuid::parse_str(&id_str).expect("Invalid UUID"),
        plant_id: Uuid::parse_str(&plant_id_str).expect("Invalid UUID"),
        timestamp: chrono::DateTime::parse_from_rfc3339(&timestamp_str)
            .expect("Invalid timestamp")
            .with_timezone(&Utc),
        care_task_ids,
        measurements,
        notes: row.get("notes"),
        photo_ids,
        created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .expect("Invalid timestamp")
            .with_timezone(&Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .expect("Invalid timestamp")
            .with_timezone(&Utc),
    }
}

pub async fn get_tracking_entries_for_plant_paginated(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
    limit: i64,
    offset: i64,
    sort_desc: bool,
    _entry_type_filter: Option<&str>,
) -> Result<TrackingEntriesResponse, AppError> {
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

    let order_clause = if sort_desc {
        "ORDER BY timestamp DESC"
    } else {
        "ORDER BY timestamp ASC"
    };

    let count_query = "SELECT COUNT(*) as count FROM tracking_entries WHERE plant_id = ?";
    let total = sqlx::query(count_query)
        .bind(plant_id.to_string())
        .fetch_one(pool)
        .await?
        .get::<i64, _>("count");

    let entries_query = format!(
        "SELECT id, plant_id, timestamp, care_task_ids, measurements, notes, photo_ids, created_at, updated_at
         FROM tracking_entries
         WHERE plant_id = ?
         {}
         LIMIT ? OFFSET ?",
        order_clause
    );

    let entries_rows = sqlx::query(&entries_query)
        .bind(plant_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let entries: Vec<TrackingEntry> = entries_rows.iter().map(row_to_entry).collect();

    Ok(TrackingEntriesResponse { entries, total })
}

#[allow(dead_code)]
pub async fn get_tracking_entries_for_plant(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
) -> Result<TrackingEntriesResponse, AppError> {
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

    let entries_rows = sqlx::query(
        "SELECT id, plant_id, timestamp, care_task_ids, measurements, notes, photo_ids, created_at, updated_at
         FROM tracking_entries
         WHERE plant_id = ?
         ORDER BY timestamp DESC"
    )
    .bind(plant_id.to_string())
    .fetch_all(pool)
    .await?;

    let entries: Vec<TrackingEntry> = entries_rows.iter().map(row_to_entry).collect();
    let total = entries.len() as i64;

    Ok(TrackingEntriesResponse { entries, total })
}

pub async fn create_tracking_entry(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
    request: &CreateTrackingEntryRequest,
) -> Result<TrackingEntry, AppError> {
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

    let care_task_ids_json = request.care_task_ids.as_ref().map(|ids| {
        serde_json::to_string(&ids.iter().map(|id| id.to_string()).collect::<Vec<_>>())
            .unwrap_or_default()
    });

    let measurements_json = request
        .measurements
        .as_ref()
        .map(|m| serde_json::to_string(m).unwrap_or_default());

    let photo_ids_json = request.photo_ids.as_ref().map(|ids| {
        serde_json::to_string(&ids.iter().map(|id| id.to_string()).collect::<Vec<_>>())
            .unwrap_or_default()
    });

    sqlx::query(
        "INSERT INTO tracking_entries (id, plant_id, timestamp, care_task_ids, measurements, notes, photo_ids, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(entry_id.to_string())
    .bind(plant_id.to_string())
    .bind(request.timestamp.to_rfc3339())
    .bind(&care_task_ids_json)
    .bind(&measurements_json)
    .bind(&request.notes)
    .bind(&photo_ids_json)
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;

    // Update last_performed for all completed care tasks
    if let Some(care_task_ids) = &request.care_task_ids {
        for task_id in care_task_ids {
            sqlx::query("UPDATE care_tasks SET last_performed = ?, updated_at = ? WHERE id = ? AND plant_id = ?")
                .bind(request.timestamp.to_rfc3339())
                .bind(now.to_rfc3339())
                .bind(task_id.to_string())
                .bind(plant_id.to_string())
                .execute(pool)
                .await?;
        }
    }

    Ok(TrackingEntry {
        id: entry_id,
        plant_id: *plant_id,
        timestamp: request.timestamp,
        care_task_ids: request.care_task_ids.clone(),
        measurements: request.measurements.clone(),
        notes: request.notes.clone(),
        photo_ids: request.photo_ids.clone(),
        created_at: now,
        updated_at: now,
    })
}

pub async fn get_tracking_entry(
    pool: &DatabasePool,
    plant_id: &Uuid,
    entry_id: &Uuid,
    user_id: &str,
) -> Result<TrackingEntry, AppError> {
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

    let row = sqlx::query(
        "SELECT id, plant_id, timestamp, care_task_ids, measurements, notes, photo_ids, created_at, updated_at
         FROM tracking_entries
         WHERE id = ? AND plant_id = ?"
    )
    .bind(entry_id.to_string())
    .bind(plant_id.to_string())
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound {
        resource: format!("Tracking entry with id {entry_id}"),
    })?;

    Ok(row_to_entry(&row))
}

pub async fn update_tracking_entry(
    pool: &DatabasePool,
    plant_id: &Uuid,
    entry_id: &Uuid,
    user_id: &str,
    request: &crate::models::tracking_entry::UpdateTrackingEntryRequest,
) -> Result<TrackingEntry, AppError> {
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
    let mut update_parts = vec!["updated_at = ?"];
    let mut values: Vec<String> = vec![now.to_rfc3339()];

    if let Some(timestamp) = &request.timestamp {
        update_parts.push("timestamp = ?");
        values.push(timestamp.to_rfc3339());
    }

    if let Some(care_task_ids) = &request.care_task_ids {
        update_parts.push("care_task_ids = ?");
        values.push(
            serde_json::to_string(&care_task_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>())
                .unwrap_or_default(),
        );
    }

    if let Some(measurements) = &request.measurements {
        update_parts.push("measurements = ?");
        values.push(serde_json::to_string(measurements).unwrap_or_default());
    }

    if let Some(notes) = &request.notes {
        update_parts.push("notes = ?");
        values.push(notes.clone());
    }

    if let Some(photo_ids) = &request.photo_ids {
        update_parts.push("photo_ids = ?");
        values.push(
            serde_json::to_string(&photo_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>())
                .unwrap_or_default(),
        );
    }

    let query = format!(
        "UPDATE tracking_entries SET {} WHERE id = ? AND plant_id = ?",
        update_parts.join(", ")
    );

    let mut query_builder = sqlx::query(&query);
    for value in values {
        query_builder = query_builder.bind(value);
    }
    query_builder = query_builder
        .bind(entry_id.to_string())
        .bind(plant_id.to_string());

    let result = query_builder.execute(pool).await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound {
            resource: format!("Tracking entry with id {entry_id}"),
        });
    }

    // Update last_performed for any new care tasks
    if let Some(care_task_ids) = &request.care_task_ids {
        let timestamp = request.timestamp.unwrap_or_else(|| now);
        for task_id in care_task_ids {
            sqlx::query("UPDATE care_tasks SET last_performed = ?, updated_at = ? WHERE id = ? AND plant_id = ?")
                .bind(timestamp.to_rfc3339())
                .bind(now.to_rfc3339())
                .bind(task_id.to_string())
                .bind(plant_id.to_string())
                .execute(pool)
                .await?;
        }
    }

    get_tracking_entry(pool, plant_id, entry_id, user_id).await
}

pub async fn delete_tracking_entry(
    pool: &DatabasePool,
    plant_id: &Uuid,
    entry_id: &Uuid,
    user_id: &str,
) -> Result<(), AppError> {
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
