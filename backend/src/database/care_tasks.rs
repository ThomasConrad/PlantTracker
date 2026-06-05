use chrono::{Duration, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::database::DatabasePool;
use crate::models::care_task::{
    CareTask, CareTaskWithStatus, CareTasksResponse, CreateCareTaskRequest, LogCareTaskRequest,
    UpdateCareTaskRequest,
};
use crate::models::tracking_entry::TrackingEntry;
use crate::utils::db_traits::{DbParse, RequireAffected};
use crate::utils::errors::AppError;

/// Convert individual field values into a CareTask. Shared by both dynamic
/// queries (via `row_to_care_task`) and compile-time-checked macro queries.
#[allow(clippy::too_many_arguments)]
fn fields_to_care_task(
    id: &str,
    plant_id: &str,
    user_id: &str,
    name: String,
    icon: Option<String>,
    color: Option<String>,
    interval_days: Option<i64>,
    amount: Option<f64>,
    unit: Option<String>,
    notes: Option<String>,
    last_performed: Option<&str>,
    sort_order: i32,
    archived_at: Option<&str>,
    created_at: &str,
    updated_at: &str,
) -> Result<CareTask, AppError> {
    Ok(CareTask {
        id: DbParse::uuid(id)?,
        plant_id: DbParse::uuid(plant_id)?,
        user_id: DbParse::uuid(user_id)?,
        name,
        icon,
        color,
        interval_days: interval_days.map(|v| v as i32),
        amount,
        unit,
        notes,
        last_performed: last_performed.map(DbParse::datetime).transpose()?,
        sort_order,
        archived_at: archived_at.map(DbParse::datetime).transpose()?,
        created_at: DbParse::datetime(created_at)?,
        updated_at: DbParse::datetime(updated_at)?,
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
    let interval_days: Option<i32> = row.try_get("interval_days").map_err(AppError::Database)?;

    fields_to_care_task(
        &id_str,
        &plant_id_str,
        &user_id_str,
        row.try_get("name").map_err(AppError::Database)?,
        row.try_get("icon").map_err(AppError::Database)?,
        row.try_get("color").map_err(AppError::Database)?,
        interval_days.map(|v| v as i64),
        row.try_get("amount").map_err(AppError::Database)?,
        row.try_get("unit").map_err(AppError::Database)?,
        row.try_get("notes").map_err(AppError::Database)?,
        last_performed_str.as_deref(),
        row.try_get("sort_order").map_err(AppError::Database)?,
        archived_at_str.as_deref(),
        &created_at_str,
        &updated_at_str,
    )
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
    crate::utils::db_traits::verify_plant_ownership(pool, plant_id, user_id).await?;

    // Dynamic query based on include_archived flag — keep as runtime query
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
    let task_id_str = task_id.to_string();
    let rec = sqlx::query!(
        r#"SELECT
            id as "id!: String",
            plant_id as "plant_id!: String",
            user_id as "user_id!: String",
            name as "name!: String",
            icon,
            color,
            interval_days,
            amount,
            unit,
            notes,
            last_performed,
            sort_order as "sort_order!: i32",
            archived_at,
            created_at as "created_at!: String",
            updated_at as "updated_at!: String"
        FROM care_tasks WHERE id = ? AND user_id = ?"#,
        task_id_str,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound {
        resource: format!("Care task with id {task_id}"),
    })?;

    let task = fields_to_care_task(
        &rec.id,
        &rec.plant_id,
        &rec.user_id,
        rec.name,
        rec.icon,
        rec.color,
        rec.interval_days,
        rec.amount,
        rec.unit,
        rec.notes,
        rec.last_performed.as_deref(),
        rec.sort_order,
        rec.archived_at.as_deref(),
        &rec.created_at,
        &rec.updated_at,
    )?;

    Ok(compute_status(task))
}

pub async fn create_care_task(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
    request: &CreateCareTaskRequest,
) -> Result<CareTaskWithStatus, AppError> {
    crate::utils::db_traits::verify_plant_ownership(pool, plant_id, user_id).await?;

    let id = Uuid::new_v4();
    let id_str = id.to_string();
    let plant_id_str = plant_id.to_string();
    let now = Utc::now().to_rfc3339();
    let last_performed = request.last_performed.map(|dt| dt.to_rfc3339());
    let sort_order = request.sort_order.unwrap_or(0);

    sqlx::query!(
        r#"INSERT INTO care_tasks (id, plant_id, user_id, name, icon, color, interval_days, amount, unit, notes, last_performed, sort_order, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        id_str,
        plant_id_str,
        user_id,
        request.name,
        request.icon,
        request.color,
        request.interval_days,
        request.amount,
        request.unit,
        request.notes,
        last_performed,
        sort_order,
        now,
        now
    )
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
    let task_id_str = task_id.to_string();
    let existing = sqlx::query!(
        r#"SELECT 1 as "x" FROM care_tasks WHERE id = ? AND user_id = ?"#,
        task_id_str,
        user_id
    )
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
    if let Some(interval) = &request.interval_days {
        let task_id_s = task_id.to_string();
        sqlx::query!(
            r#"UPDATE care_tasks SET interval_days = ?, updated_at = ? WHERE id = ? AND user_id = ?"#,
            *interval,
            now,
            task_id_s,
            user_id
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    }

    if let Some(amount) = &request.amount {
        let task_id_s = task_id.to_string();
        sqlx::query!(
            r#"UPDATE care_tasks SET amount = ?, updated_at = ? WHERE id = ? AND user_id = ?"#,
            *amount,
            now,
            task_id_s,
            user_id
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    }

    if let Some(sort_order) = request.sort_order {
        let task_id_s = task_id.to_string();
        sqlx::query!(
            r#"UPDATE care_tasks SET sort_order = ?, updated_at = ? WHERE id = ? AND user_id = ?"#,
            sort_order,
            now,
            task_id_s,
            user_id
        )
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
    let task_id_str = task_id.to_string();
    sqlx::query!(
        r#"DELETE FROM care_tasks WHERE id = ? AND user_id = ?"#,
        task_id_str,
        user_id
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?
    .require_affected(format!("Care task with id {task_id}"))?;

    Ok(())
}

pub async fn archive_care_task(
    pool: &DatabasePool,
    task_id: &Uuid,
    user_id: &str,
) -> Result<CareTaskWithStatus, AppError> {
    let now = Utc::now().to_rfc3339();
    let task_id_str = task_id.to_string();
    sqlx::query!(
        r#"UPDATE care_tasks SET archived_at = ?, updated_at = ? WHERE id = ? AND user_id = ? AND archived_at IS NULL"#,
        now,
        now,
        task_id_str,
        user_id
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?
    .require_affected(format!("Care task with id {task_id}"))?;

    get_care_task(pool, task_id, user_id).await
}

pub async fn unarchive_care_task(
    pool: &DatabasePool,
    task_id: &Uuid,
    user_id: &str,
) -> Result<CareTaskWithStatus, AppError> {
    let now = Utc::now().to_rfc3339();
    let task_id_str = task_id.to_string();
    sqlx::query!(
        r#"UPDATE care_tasks SET archived_at = NULL, updated_at = ? WHERE id = ? AND user_id = ?"#,
        now,
        task_id_str,
        user_id
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?
    .require_affected(format!("Care task with id {task_id}"))?;

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
        let sort_order = i as i32;
        let task_id_str = task_id.to_string();
        let plant_id_str = plant_id.to_string();
        sqlx::query!(
            r#"UPDATE care_tasks SET sort_order = ?, updated_at = ? WHERE id = ? AND plant_id = ? AND user_id = ?"#,
            sort_order,
            now,
            task_id_str,
            plant_id_str,
            user_id
        )
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
    let entry_id_str = entry_id.to_string();
    let plant_id_str = task.plant_id.to_string();
    let timestamp_str = timestamp.to_rfc3339();
    let now_str = now.to_rfc3339();
    sqlx::query!(
        r#"INSERT INTO tracking_entries (id, plant_id, timestamp, care_task_ids, notes, photo_ids, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        entry_id_str,
        plant_id_str,
        timestamp_str,
        care_task_ids_json,
        request.notes,
        photo_ids_json,
        now_str,
        now_str
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    // Update last_performed on care task
    let task_id_str = task_id.to_string();
    sqlx::query!(
        r#"UPDATE care_tasks SET last_performed = ?, updated_at = ? WHERE id = ?"#,
        timestamp_str,
        now_str,
        task_id_str
    )
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
    let recs = sqlx::query!(
        r#"SELECT
            id as "id!: String",
            plant_id as "plant_id!: String",
            user_id as "user_id!: String",
            name as "name!: String",
            icon,
            color,
            interval_days,
            amount,
            unit,
            notes,
            last_performed,
            sort_order as "sort_order!: i32",
            archived_at,
            created_at as "created_at!: String",
            updated_at as "updated_at!: String"
        FROM care_tasks WHERE user_id = ? AND archived_at IS NULL AND interval_days IS NOT NULL"#,
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut tasks = Vec::with_capacity(recs.len());
    for rec in recs {
        let task = fields_to_care_task(
            &rec.id,
            &rec.plant_id,
            &rec.user_id,
            rec.name,
            rec.icon,
            rec.color,
            rec.interval_days,
            rec.amount,
            rec.unit,
            rec.notes,
            rec.last_performed.as_deref(),
            rec.sort_order,
            rec.archived_at.as_deref(),
            &rec.created_at,
            &rec.updated_at,
        )?;
        tasks.push(compute_status(task));
    }

    Ok(tasks)
}
