use chrono::Utc;
use sqlx::SqlitePool;

use crate::utils::errors::{AppError, Result};

/// A record tracking a synced Google Task
#[derive(Debug, Clone)]
pub struct GoogleTaskSyncRecord {
    pub id: String,
    pub user_id: String,
    pub plant_id: String,
    pub care_task_id: String,
    pub google_task_id: String,
    pub google_task_list_id: String,
    pub due_date: String,
    pub status: String,
    pub synced_at: String,
    pub completed_at: Option<String>,
}

/// Check if a task for a specific care_task + due_date already exists
pub async fn get_synced_task(
    pool: &SqlitePool,
    user_id: &str,
    care_task_id: &str,
    due_date: &str,
) -> Result<Option<GoogleTaskSyncRecord>> {
    let row = sqlx::query_as!(
        GoogleTaskSyncRecord,
        r#"
        SELECT
            id as "id!: String",
            user_id as "user_id!: String",
            plant_id as "plant_id!: String",
            care_task_id as "care_task_id!: String",
            google_task_id as "google_task_id!: String",
            google_task_list_id as "google_task_list_id!: String",
            due_date as "due_date!: String",
            status as "status!: String",
            synced_at as "synced_at!: String",
            completed_at as "completed_at?: String"
        FROM google_task_sync
        WHERE user_id = ? AND care_task_id = ? AND due_date = ?
        "#,
        user_id,
        care_task_id,
        due_date
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(row)
}

/// Insert a new synced task record
pub async fn insert_synced_task(
    pool: &SqlitePool,
    user_id: &str,
    plant_id: &str,
    care_task_id: &str,
    google_task_id: &str,
    google_task_list_id: &str,
    due_date: &str,
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO google_task_sync (
            user_id, plant_id, care_task_id, google_task_id,
            google_task_list_id, due_date, status
        ) VALUES (?, ?, ?, ?, ?, ?, 'needsAction')
        "#,
        user_id,
        plant_id,
        care_task_id,
        google_task_id,
        google_task_list_id,
        due_date
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(())
}

/// Get all incomplete synced tasks for a user (for polling completions)
pub async fn get_incomplete_tasks(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<GoogleTaskSyncRecord>> {
    let rows = sqlx::query_as!(
        GoogleTaskSyncRecord,
        r#"
        SELECT
            id as "id!: String",
            user_id as "user_id!: String",
            plant_id as "plant_id!: String",
            care_task_id as "care_task_id!: String",
            google_task_id as "google_task_id!: String",
            google_task_list_id as "google_task_list_id!: String",
            due_date as "due_date!: String",
            status as "status!: String",
            synced_at as "synced_at!: String",
            completed_at as "completed_at?: String"
        FROM google_task_sync
        WHERE user_id = ? AND status = 'needsAction'
        ORDER BY due_date ASC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(rows)
}

/// Mark a synced task as completed
pub async fn mark_task_completed(
    pool: &SqlitePool,
    google_task_id: &str,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();

    sqlx::query!(
        r#"
        UPDATE google_task_sync
        SET status = 'completed', completed_at = ?, updated_at = datetime('now')
        WHERE google_task_id = ? AND status = 'needsAction'
        "#,
        now,
        google_task_id
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(())
}

/// Delete all sync records for a user (on disconnect)
pub async fn delete_all_for_user(pool: &SqlitePool, user_id: &str) -> Result<()> {
    sqlx::query!("DELETE FROM google_task_sync WHERE user_id = ?", user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

    Ok(())
}

/// Delete sync records for tasks whose due_date has passed (cleanup)
pub async fn cleanup_old_tasks(pool: &SqlitePool, before_date: &str) -> Result<u64> {
    let result = sqlx::query!(
        "DELETE FROM google_task_sync WHERE due_date < ? AND status = 'completed'",
        before_date
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(result.rows_affected())
}
