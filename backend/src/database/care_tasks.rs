use chrono::{DateTime, Duration, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::database::DatabasePool;
use crate::models::care_task::{
    CareTask, CareTaskWithStatus, CareTasksResponse, CreateCareTaskRequest, LogCareTaskRequest,
    UpdateCareTaskRequest,
};
use crate::models::tracking_entry::TrackingEntry;
use crate::utils::errors::AppError;

fn parse_dt(s: &str) -> Result<DateTime<Utc>, AppError> {
    s.parse::<DateTime<Utc>>().map_err(|_| AppError::Internal {
        message: "Invalid datetime in database".to_string(),
    })
}

fn row_to_care_task(row: &sqlx::sqlite::SqliteRow) -> Result<CareTask, AppError> {
    let id_str: String = row.try_get("id").map_err(AppError::Database)?;
    let plant_id_str: String = row.try_get("plant_id").map_err(AppError::Database)?;
    let user_id_str: String = row.try_get("user_id").map_err(AppError::Database)?;
    let created_at_str: String = row.try_get("created_at").map_err(AppError::Database)?;
    let updated_at_str: String = row.try_get("updated_at").map_err(AppError::Database)?;
    let last_performed_str: Option<String> =
        row.try_get("last_performed").map_err(AppError::Database)?;
    let archived_at_str: Option<String> = row.try_get("archived_at").map_err(AppError::Database)?;

    Ok(CareTask {
        id: Uuid::parse_str(&id_str).map_err(|_| AppError::Internal {
            message: "Invalid UUID in database".to_string(),
        })?,
        plant_id: Uuid::parse_str(&plant_id_str).map_err(|_| AppError::Internal {
            message: "Invalid UUID in database".to_string(),
        })?,
        user_id: Uuid::parse_str(&user_id_str).map_err(|_| AppError::Internal {
            message: "Invalid UUID in database".to_string(),
        })?,
        name: row.try_get("name").map_err(AppError::Database)?,
        icon: row.try_get("icon").map_err(AppError::Database)?,
        color: row.try_get("color").map_err(AppError::Database)?,
        interval_days: row.try_get("interval_days").map_err(AppError::Database)?,
        amount: row.try_get("amount").map_err(AppError::Database)?,
        unit: row.try_get("unit").map_err(AppError::Database)?,
        notes: row.try_get("notes").map_err(AppError::Database)?,
        last_performed: last_performed_str.map(|s| parse_dt(&s)).transpose()?,
        sort_order: row.try_get("sort_order").map_err(AppError::Database)?,
        archived_at: archived_at_str.map(|s| parse_dt(&s)).transpose()?,
        created_at: parse_dt(&created_at_str)?,
        updated_at: parse_dt(&updated_at_str)?,
    })
}

fn compute_status(task: CareTask) -> CareTaskWithStatus {
    let now = Utc::now();
    let (next_due, days_overdue, is_due) = match (task.interval_days, &task.last_performed) {
        (Some(interval), Some(last)) => {
            let due = *last + Duration::days(i64::from(interval));
            let diff = (now - due).num_days();
            (Some(due), Some(diff), diff >= 0)
        }
        (Some(interval), None) => {
            // Never performed — due since creation + interval
            let due = task.created_at + Duration::days(i64::from(interval));
            let diff = (now - due).num_days();
            (Some(due), Some(diff), diff >= 0)
        }
        (None, _) => (None, None, false),
    };

    CareTaskWithStatus {
        task,
        next_due,
        days_overdue,
        is_due,
    }
}

pub async fn list_care_tasks_for_plant(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
    include_archived: bool,
) -> Result<CareTasksResponse, AppError> {
    // Verify plant ownership
    let plant_exists = sqlx::query("SELECT 1 FROM plants WHERE id = ? AND user_id = ?")
        .bind(plant_id.to_string())
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

    if plant_exists.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    let query = if include_archived {
        "SELECT * FROM care_tasks WHERE plant_id = ? AND user_id = ? ORDER BY sort_order ASC, created_at ASC"
    } else {
        "SELECT * FROM care_tasks WHERE plant_id = ? AND user_id = ? AND archived_at IS NULL ORDER BY sort_order ASC, created_at ASC"
    };

    let rows = sqlx::query(query)
        .bind(plant_id.to_string())
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

    let tasks = rows
        .iter()
        .map(row_to_care_task)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(compute_status)
        .collect();

    Ok(CareTasksResponse { tasks })
}

pub async fn get_care_task(
    pool: &DatabasePool,
    task_id: &Uuid,
    user_id: &str,
) -> Result<CareTaskWithStatus, AppError> {
    let row = sqlx::query("SELECT * FROM care_tasks WHERE id = ? AND user_id = ?")
        .bind(task_id.to_string())
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound {
            resource: format!("Care task with id {task_id}"),
        })?;

    let task = row_to_care_task(&row)?;
    Ok(compute_status(task))
}

pub async fn create_care_task(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
    request: &CreateCareTaskRequest,
) -> Result<CareTaskWithStatus, AppError> {
    // Verify plant ownership
    let plant_exists = sqlx::query("SELECT 1 FROM plants WHERE id = ? AND user_id = ?")
        .bind(plant_id.to_string())
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

    if plant_exists.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    let id = Uuid::new_v4();
    let now = Utc::now().to_rfc3339();
    let last_performed = request.last_performed.map(|dt| dt.to_rfc3339());
    let sort_order = request.sort_order.unwrap_or(0);

    sqlx::query(
        "INSERT INTO care_tasks (id, plant_id, user_id, name, icon, color, interval_days, amount, unit, notes, last_performed, sort_order, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(id.to_string())
    .bind(plant_id.to_string())
    .bind(user_id)
    .bind(&request.name)
    .bind(&request.icon)
    .bind(&request.color)
    .bind(request.interval_days)
    .bind(request.amount)
    .bind(&request.unit)
    .bind(&request.notes)
    .bind(&last_performed)
    .bind(sort_order)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    get_care_task(pool, &id, user_id).await
}

pub async fn update_care_task(
    pool: &DatabasePool,
    task_id: &Uuid,
    user_id: &str,
    request: &UpdateCareTaskRequest,
) -> Result<CareTaskWithStatus, AppError> {
    // Verify ownership
    let existing = sqlx::query("SELECT 1 FROM care_tasks WHERE id = ? AND user_id = ?")
        .bind(task_id.to_string())
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

    if existing.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Care task with id {task_id}"),
        });
    }

    let now = Utc::now().to_rfc3339();

    // Build dynamic update
    let mut sets = vec!["updated_at = ?"];
    let mut binds: Vec<Option<String>> = vec![Some(now.clone())];

    if let Some(name) = &request.name {
        sets.push("name = ?");
        binds.push(Some(name.clone()));
    }
    if let Some(icon) = &request.icon {
        sets.push("icon = ?");
        binds.push(icon.clone());
    }
    if let Some(color) = &request.color {
        sets.push("color = ?");
        binds.push(color.clone());
    }
    if let Some(notes) = &request.notes {
        sets.push("notes = ?");
        binds.push(notes.clone());
    }
    if let Some(unit) = &request.unit {
        sets.push("unit = ?");
        binds.push(unit.clone());
    }
    if let Some(last_performed) = &request.last_performed {
        sets.push("last_performed = ?");
        binds.push(last_performed.map(|dt| dt.to_rfc3339()));
    }

    let query_str = format!(
        "UPDATE care_tasks SET {} WHERE id = ? AND user_id = ?",
        sets.join(", ")
    );

    let mut query = sqlx::query(&query_str);
    for bind in &binds {
        query = query.bind(bind);
    }

    // Handle numeric fields separately since they're not Option<String>
    // Actually we need a different approach for mixed types. Let's use a simpler strategy:
    // Re-fetch and apply changes in Rust, then write back.

    // For interval_days, amount, sort_order we need separate queries if provided
    if let Some(interval) = &request.interval_days {
        sqlx::query(
            "UPDATE care_tasks SET interval_days = ?, updated_at = ? WHERE id = ? AND user_id = ?",
        )
        .bind(*interval)
        .bind(&now)
        .bind(task_id.to_string())
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    }

    if let Some(amount) = &request.amount {
        sqlx::query(
            "UPDATE care_tasks SET amount = ?, updated_at = ? WHERE id = ? AND user_id = ?",
        )
        .bind(*amount)
        .bind(&now)
        .bind(task_id.to_string())
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    }

    if let Some(sort_order) = request.sort_order {
        sqlx::query(
            "UPDATE care_tasks SET sort_order = ?, updated_at = ? WHERE id = ? AND user_id = ?",
        )
        .bind(sort_order)
        .bind(&now)
        .bind(task_id.to_string())
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    }

    // Execute the string-fields query
    if sets.len() > 1 {
        // More than just updated_at
        query = query.bind(task_id.to_string()).bind(user_id);
        query.execute(pool).await.map_err(AppError::Database)?;
    }

    get_care_task(pool, task_id, user_id).await
}

pub async fn delete_care_task(
    pool: &DatabasePool,
    task_id: &Uuid,
    user_id: &str,
) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM care_tasks WHERE id = ? AND user_id = ?")
        .bind(task_id.to_string())
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound {
            resource: format!("Care task with id {task_id}"),
        });
    }

    Ok(())
}

pub async fn archive_care_task(
    pool: &DatabasePool,
    task_id: &Uuid,
    user_id: &str,
) -> Result<CareTaskWithStatus, AppError> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "UPDATE care_tasks SET archived_at = ?, updated_at = ? WHERE id = ? AND user_id = ? AND archived_at IS NULL"
    )
    .bind(&now)
    .bind(&now)
    .bind(task_id.to_string())
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound {
            resource: format!("Care task with id {task_id}"),
        });
    }

    get_care_task(pool, task_id, user_id).await
}

pub async fn unarchive_care_task(
    pool: &DatabasePool,
    task_id: &Uuid,
    user_id: &str,
) -> Result<CareTaskWithStatus, AppError> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "UPDATE care_tasks SET archived_at = NULL, updated_at = ? WHERE id = ? AND user_id = ?",
    )
    .bind(&now)
    .bind(task_id.to_string())
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound {
            resource: format!("Care task with id {task_id}"),
        });
    }

    get_care_task(pool, task_id, user_id).await
}

pub async fn reorder_care_tasks(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
    task_ids: &[Uuid],
) -> Result<CareTasksResponse, AppError> {
    let now = Utc::now().to_rfc3339();

    for (i, task_id) in task_ids.iter().enumerate() {
        sqlx::query(
            "UPDATE care_tasks SET sort_order = ?, updated_at = ? WHERE id = ? AND plant_id = ? AND user_id = ?"
        )
        .bind(i as i32)
        .bind(&now)
        .bind(task_id.to_string())
        .bind(plant_id.to_string())
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    }

    list_care_tasks_for_plant(pool, plant_id, user_id, false).await
}

/// Log a care task completion: creates a tracking entry and updates last_performed
pub async fn log_care_task(
    pool: &DatabasePool,
    task_id: &Uuid,
    user_id: &str,
    request: &LogCareTaskRequest,
) -> Result<(CareTaskWithStatus, TrackingEntry), AppError> {
    // Get the care task (verifies ownership)
    let task_with_status = get_care_task(pool, task_id, user_id).await?;
    let task = &task_with_status.task;

    let now = Utc::now();
    let timestamp = request.timestamp.unwrap_or(now);
    let entry_id = Uuid::new_v4();

    let care_task_ids_json = serde_json::to_string(&vec![task_id.to_string()]).unwrap_or_default();

    let photo_ids_json = request.photo_ids.as_ref().map(|ids| {
        serde_json::to_string(&ids.iter().map(|id| id.to_string()).collect::<Vec<_>>())
            .unwrap_or_default()
    });

    // Create tracking entry
    sqlx::query(
        "INSERT INTO tracking_entries (id, plant_id, timestamp, care_task_ids, notes, photo_ids, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(entry_id.to_string())
    .bind(task.plant_id.to_string())
    .bind(timestamp.to_rfc3339())
    .bind(&care_task_ids_json)
    .bind(&request.notes)
    .bind(&photo_ids_json)
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    // Update last_performed on care task
    sqlx::query("UPDATE care_tasks SET last_performed = ?, updated_at = ? WHERE id = ?")
        .bind(timestamp.to_rfc3339())
        .bind(now.to_rfc3339())
        .bind(task_id.to_string())
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

    let entry = TrackingEntry {
        id: entry_id,
        plant_id: task.plant_id,
        timestamp,
        care_task_ids: Some(vec![*task_id]),
        measurements: None,
        notes: request.notes.clone(),
        photo_ids: request.photo_ids.clone(),
        created_at: now,
        updated_at: now,
    };

    // Re-fetch task with updated status
    let updated_task = get_care_task(pool, task_id, user_id).await?;

    Ok((updated_task, entry))
}

/// Get all care tasks with schedules for a user (for reminder calculation)
pub async fn list_all_scheduled_care_tasks_for_user(
    pool: &DatabasePool,
    user_id: &str,
) -> Result<Vec<CareTaskWithStatus>, AppError> {
    let rows = sqlx::query(
        "SELECT * FROM care_tasks WHERE user_id = ? AND archived_at IS NULL AND interval_days IS NOT NULL"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    rows.iter()
        .map(row_to_care_task)
        .collect::<Result<Vec<_>, _>>()
        .map(|tasks| tasks.into_iter().map(compute_status).collect())
}
