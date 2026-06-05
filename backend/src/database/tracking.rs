use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use crate::database::DatabasePool;
use crate::models::tracking_entry::{
    CreateTrackingEntryRequest, Measurement, TrackingEntriesResponse, TrackingEntry,
};
use crate::utils::db_traits::{DbParse, RequireAffected};
use crate::utils::errors::AppError;

/// Convert raw field values into a TrackingEntry. Shared logic for both
/// dynamic queries (via row_to_entry) and macro queries.
#[allow(clippy::too_many_arguments)]
fn fields_to_entry(
    id: &str,
    plant_id: &str,
    timestamp: &str,
    care_task_ids_json: Option<&str>,
    measurements_json: Option<&str>,
    notes: Option<String>,
    photo_ids_json: Option<&str>,
    created_at: &str,
    updated_at: &str,
) -> Result<TrackingEntry, AppError> {
    let care_task_ids = care_task_ids_json.and_then(|s| {
        serde_json::from_str::<Vec<String>>(s).ok().map(|ids| {
            ids.iter()
                .filter_map(|id| Uuid::parse_str(id).ok())
                .collect()
        })
    });

    let measurements: Option<Vec<Measurement>> =
        measurements_json.and_then(|s| serde_json::from_str(s).ok());

    let photo_ids = photo_ids_json.and_then(|s| {
        serde_json::from_str::<Vec<String>>(s).ok().map(|ids| {
            ids.iter()
                .filter_map(|id| Uuid::parse_str(id).ok())
                .collect()
        })
    });

    Ok(TrackingEntry {
        id: DbParse::uuid(id)?,
        plant_id: DbParse::uuid(plant_id)?,
        timestamp: DbParse::datetime(timestamp)?,
        care_task_ids,
        measurements,
        notes,
        photo_ids,
        created_at: DbParse::datetime(created_at)?,
        updated_at: DbParse::datetime(updated_at)?,
    })
}

/// Convert a SqliteRow to TrackingEntry (used by dynamic queries)
fn row_to_entry(row: &sqlx::sqlite::SqliteRow) -> Result<TrackingEntry, AppError> {
    let id: String = row.get("id");
    let plant_id: String = row.get("plant_id");
    let timestamp: String = row.get("timestamp");
    let care_task_ids: Option<String> = row.get("care_task_ids");
    let measurements: Option<String> = row.get("measurements");
    let notes: Option<String> = row.get("notes");
    let photo_ids: Option<String> = row.get("photo_ids");
    let created_at: String = row.get("created_at");
    let updated_at: String = row.get("updated_at");

    fields_to_entry(
        &id,
        &plant_id,
        &timestamp,
        care_task_ids.as_deref(),
        measurements.as_deref(),
        notes,
        photo_ids.as_deref(),
        &created_at,
        &updated_at,
    )
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
    crate::utils::db_traits::verify_plant_ownership(pool, plant_id, user_id).await?;

    let plant_id_str = plant_id.to_string();

    let order_clause = if sort_desc {
        "ORDER BY timestamp DESC"
    } else {
        "ORDER BY timestamp ASC"
    };

    let count_query = "SELECT COUNT(*) as count FROM tracking_entries WHERE plant_id = ?";
    let total = sqlx::query(count_query)
        .bind(&plant_id_str)
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
        .bind(&plant_id_str)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let entries: Vec<TrackingEntry> = entries_rows
        .iter()
        .map(row_to_entry)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(TrackingEntriesResponse { entries, total })
}

#[allow(dead_code)]
pub async fn get_tracking_entries_for_plant(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
) -> Result<TrackingEntriesResponse, AppError> {
    crate::utils::db_traits::verify_plant_ownership(pool, plant_id, user_id).await?;

    let plant_id_str = plant_id.to_string();

    let entries_rows = sqlx::query!(
        r#"SELECT id, plant_id, timestamp, care_task_ids, measurements, notes, photo_ids, created_at, updated_at
         FROM tracking_entries
         WHERE plant_id = ?
         ORDER BY timestamp DESC"#,
        plant_id_str
    )
    .fetch_all(pool)
    .await?;

    let entries: Vec<TrackingEntry> = entries_rows
        .iter()
        .map(|row| {
            fields_to_entry(
                &row.id,
                &row.plant_id,
                &row.timestamp,
                row.care_task_ids.as_deref(),
                row.measurements.as_deref(),
                row.notes.clone(),
                row.photo_ids.as_deref(),
                &row.created_at,
                &row.updated_at,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let total = entries.len() as i64;

    Ok(TrackingEntriesResponse { entries, total })
}

pub async fn create_tracking_entry(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
    request: &CreateTrackingEntryRequest,
) -> Result<TrackingEntry, AppError> {
    crate::utils::db_traits::verify_plant_ownership(pool, plant_id, user_id).await?;

    let plant_id_str = plant_id.to_string();

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

    let entry_id_str = entry_id.to_string();
    let timestamp_str = request.timestamp.to_rfc3339();
    let now_str = now.to_rfc3339();

    sqlx::query!(
        r#"INSERT INTO tracking_entries (id, plant_id, timestamp, care_task_ids, measurements, notes, photo_ids, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        entry_id_str,
        plant_id_str,
        timestamp_str,
        care_task_ids_json,
        measurements_json,
        request.notes,
        photo_ids_json,
        now_str,
        now_str
    )
    .execute(pool)
    .await?;

    // Update last_performed for all completed care tasks
    if let Some(care_task_ids) = &request.care_task_ids {
        for task_id in care_task_ids {
            let task_id_str = task_id.to_string();
            sqlx::query!(
                r#"UPDATE care_tasks SET last_performed = ?, updated_at = ? WHERE id = ? AND plant_id = ?"#,
                timestamp_str,
                now_str,
                task_id_str,
                plant_id_str
            )
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
    crate::utils::db_traits::verify_plant_ownership(pool, plant_id, user_id).await?;

    let plant_id_str = plant_id.to_string();
    let entry_id_str = entry_id.to_string();

    let row = sqlx::query!(
        r#"SELECT id, plant_id, timestamp, care_task_ids, measurements, notes, photo_ids, created_at, updated_at
         FROM tracking_entries
         WHERE id = ? AND plant_id = ?"#,
        entry_id_str,
        plant_id_str
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound {
        resource: format!("Tracking entry with id {entry_id}"),
    })?;

    fields_to_entry(
        &row.id,
        &row.plant_id,
        &row.timestamp,
        row.care_task_ids.as_deref(),
        row.measurements.as_deref(),
        row.notes.clone(),
        row.photo_ids.as_deref(),
        &row.created_at,
        &row.updated_at,
    )
}

pub async fn update_tracking_entry(
    pool: &DatabasePool,
    plant_id: &Uuid,
    entry_id: &Uuid,
    user_id: &str,
    request: &crate::models::tracking_entry::UpdateTrackingEntryRequest,
) -> Result<TrackingEntry, AppError> {
    crate::utils::db_traits::verify_plant_ownership(pool, plant_id, user_id).await?;

    let plant_id_str = plant_id.to_string();
    let entry_id_str = entry_id.to_string();

    let entry_exists = sqlx::query!(
        r#"SELECT 1 as "x" FROM tracking_entries WHERE id = ? AND plant_id = ?"#,
        entry_id_str,
        plant_id_str
    )
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
            serde_json::to_string(
                &care_task_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>(),
            )
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
            serde_json::to_string(
                &photo_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>(),
            )
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

    query_builder
        .execute(pool)
        .await?
        .require_affected(format!("Tracking entry with id {entry_id}"))?;

    // Update last_performed for any new care tasks
    if let Some(care_task_ids) = &request.care_task_ids {
        let timestamp = request.timestamp.unwrap_or(now);
        let timestamp_str = timestamp.to_rfc3339();
        let now_str = now.to_rfc3339();
        for task_id in care_task_ids {
            let task_id_str = task_id.to_string();
            sqlx::query!(
                r#"UPDATE care_tasks SET last_performed = ?, updated_at = ? WHERE id = ? AND plant_id = ?"#,
                timestamp_str,
                now_str,
                task_id_str,
                plant_id_str
            )
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
    crate::utils::db_traits::verify_plant_ownership(pool, plant_id, user_id).await?;

    let plant_id_str = plant_id.to_string();
    let entry_id_str = entry_id.to_string();

    sqlx::query!(
        r#"DELETE FROM tracking_entries WHERE id = ? AND plant_id = ?"#,
        entry_id_str,
        plant_id_str
    )
    .execute(pool)
    .await?
    .require_affected(format!("Tracking entry with id {entry_id}"))?;

    Ok(())
}
