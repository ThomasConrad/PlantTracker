use chrono::Utc;
use uuid::Uuid;

use crate::database::care_tasks as db_care_tasks;
use crate::database::DatabasePool;
use crate::models::{DueReminder, ReminderPreferences, UpdateReminderPreferencesRequest};
use crate::utils::errors::{AppError, Result};

pub async fn get_or_create_preferences(
    pool: &DatabasePool,
    user_id: &str,
) -> Result<ReminderPreferences> {
    let now = Utc::now().to_rfc3339();

    sqlx::query!(
        "INSERT INTO user_reminder_preferences (user_id, enabled, reminder_time, timezone, browser_notifications_enabled, created_at, updated_at)
         VALUES (?, FALSE, '09:00', 'UTC', FALSE, ?, ?)
         ON CONFLICT(user_id) DO NOTHING",
        user_id,
        now,
        now,
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    let row = sqlx::query!(
        "SELECT enabled, reminder_time, timezone, browser_notifications_enabled,
                push_health_alerts, push_daily_summary, push_coach_suggestions, push_reminders
         FROM user_reminder_preferences
         WHERE user_id = ?",
        user_id,
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(ReminderPreferences {
        enabled: row.enabled,
        reminder_time: row.reminder_time,
        timezone: row.timezone,
        browser_notifications_enabled: row.browser_notifications_enabled,
        push_health_alerts: row.push_health_alerts,
        push_daily_summary: row.push_daily_summary,
        push_coach_suggestions: row.push_coach_suggestions,
        push_reminders: row.push_reminders,
    })
}

pub async fn update_preferences(
    pool: &DatabasePool,
    user_id: &str,
    request: &UpdateReminderPreferencesRequest,
) -> Result<ReminderPreferences> {
    let now = Utc::now().to_rfc3339();

    sqlx::query!(
        "INSERT INTO user_reminder_preferences (user_id, enabled, reminder_time, timezone, browser_notifications_enabled,
            push_health_alerts, push_daily_summary, push_coach_suggestions, push_reminders, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(user_id) DO UPDATE
         SET enabled = excluded.enabled,
             reminder_time = excluded.reminder_time,
             timezone = excluded.timezone,
             browser_notifications_enabled = excluded.browser_notifications_enabled,
             push_health_alerts = excluded.push_health_alerts,
             push_daily_summary = excluded.push_daily_summary,
             push_coach_suggestions = excluded.push_coach_suggestions,
             push_reminders = excluded.push_reminders,
             updated_at = excluded.updated_at",
        user_id,
        request.enabled,
        request.reminder_time,
        request.timezone,
        request.browser_notifications_enabled,
        request.push_health_alerts,
        request.push_daily_summary,
        request.push_coach_suggestions,
        request.push_reminders,
        now,
        now,
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    get_or_create_preferences(pool, user_id).await
}

/// List all due reminders by iterating over care_tasks with intervals
pub async fn list_due_reminders(pool: &DatabasePool, user_id: &str) -> Result<Vec<DueReminder>> {
    let tasks = db_care_tasks::list_all_scheduled_care_tasks_for_user(pool, user_id).await?;

    let now = Utc::now();
    let mut due = Vec::new();

    for task_with_status in &tasks {
        if !task_with_status.is_due {
            continue;
        }

        let task = &task_with_status.task;
        let due_at = task_with_status.next_due.unwrap_or(now);
        let due_date = due_at.format("%Y-%m-%d").to_string();
        let days_overdue = task_with_status.days_overdue.unwrap_or(0);

        // Check if already sent
        let already_sent = reminder_delivery_exists(
            pool,
            user_id,
            &task.plant_id.to_string(),
            &task.id.to_string(),
            &due_date,
        )
        .await?;

        // Get plant name
        let plant_name: String = sqlx::query_scalar("SELECT name FROM plants WHERE id = ?")
            .bind(task.plant_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?
            .unwrap_or_else(|| "Unknown".to_string());

        due.push(DueReminder {
            plant_id: task.plant_id.to_string(),
            plant_name,
            care_task_id: task.id.to_string(),
            care_task_name: task.name.clone(),
            reminder_type: task.name.to_lowercase(), // for backward compat
            due_at: due_at.to_rfc3339(),
            due_date,
            days_overdue,
            already_sent,
        });
    }

    due.sort_by(|a, b| b.days_overdue.cmp(&a.days_overdue));
    Ok(due)
}

pub async fn dispatch_due_reminders(
    pool: &DatabasePool,
    user_id: &str,
) -> Result<Vec<DueReminder>> {
    let due = list_due_reminders(pool, user_id).await?;
    let now = Utc::now().to_rfc3339();
    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    let mut sent = Vec::new();

    for reminder in due {
        let id = Uuid::new_v4().to_string();

        sqlx::query!(
            "INSERT INTO reminder_deliveries
             (id, user_id, plant_id, care_task_id, due_date, due_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(user_id, plant_id, care_task_id, due_date)
             DO UPDATE SET updated_at = excluded.updated_at",
            id,
            user_id,
            reminder.plant_id,
            reminder.care_task_id,
            reminder.due_date,
            reminder.due_at,
            now,
            now,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;

        let result = sqlx::query!(
            "UPDATE reminder_deliveries
             SET sent_at = ?, updated_at = ?
             WHERE user_id = ?
               AND plant_id = ?
               AND care_task_id = ?
               AND due_date = ?
               AND sent_at IS NULL",
            now,
            now,
            user_id,
            reminder.plant_id,
            reminder.care_task_id,
            reminder.due_date,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;

        if result.rows_affected() == 1 {
            sent.push(DueReminder {
                already_sent: true,
                ..reminder
            });
        }
    }

    tx.commit().await.map_err(AppError::Database)?;
    Ok(sent)
}

async fn reminder_delivery_exists(
    pool: &DatabasePool,
    user_id: &str,
    plant_id: &str,
    care_task_id: &str,
    due_date: &str,
) -> Result<bool> {
    let sent = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) as count
         FROM reminder_deliveries
         WHERE user_id = ?
           AND plant_id = ?
           AND care_task_id = ?
           AND due_date = ?
           AND sent_at IS NOT NULL",
    )
    .bind(user_id)
    .bind(plant_id)
    .bind(care_task_id)
    .bind(due_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(sent > 0)
}
