use chrono::{DateTime, Duration, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::database::DatabasePool;
use crate::models::{DueReminder, ReminderPreferences, UpdateReminderPreferencesRequest};
use crate::utils::errors::{AppError, Result};

#[derive(Debug)]
struct PlantReminderRow {
    plant_id: String,
    plant_name: String,
    created_at: String,
    last_watered: Option<String>,
    last_fertilized: Option<String>,
    watering_interval_days: Option<i32>,
    fertilizing_interval_days: Option<i32>,
}

fn parse_dt(value: &str) -> Result<DateTime<Utc>> {
    value
        .parse::<DateTime<Utc>>()
        .map_err(|_| AppError::Internal {
            message: "Invalid datetime in database".to_string(),
        })
}

pub async fn get_or_create_preferences(
    pool: &DatabasePool,
    user_id: &str,
) -> Result<ReminderPreferences> {
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO user_reminder_preferences (user_id, enabled, reminder_time, timezone, browser_notifications_enabled, created_at, updated_at)
         VALUES (?, FALSE, '09:00', 'UTC', FALSE, ?, ?)
         ON CONFLICT(user_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    let row = sqlx::query(
        "SELECT enabled, reminder_time, timezone, browser_notifications_enabled
         FROM user_reminder_preferences
         WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(ReminderPreferences {
        enabled: row
            .try_get::<bool, _>("enabled")
            .map_err(AppError::Database)?,
        reminder_time: row
            .try_get::<String, _>("reminder_time")
            .map_err(AppError::Database)?,
        timezone: row
            .try_get::<String, _>("timezone")
            .map_err(AppError::Database)?,
        browser_notifications_enabled: row
            .try_get::<bool, _>("browser_notifications_enabled")
            .map_err(AppError::Database)?,
    })
}

pub async fn update_preferences(
    pool: &DatabasePool,
    user_id: &str,
    request: &UpdateReminderPreferencesRequest,
) -> Result<ReminderPreferences> {
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO user_reminder_preferences (user_id, enabled, reminder_time, timezone, browser_notifications_enabled, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(user_id) DO UPDATE
         SET enabled = excluded.enabled,
             reminder_time = excluded.reminder_time,
             timezone = excluded.timezone,
             browser_notifications_enabled = excluded.browser_notifications_enabled,
             updated_at = excluded.updated_at",
    )
    .bind(user_id)
    .bind(request.enabled)
    .bind(&request.reminder_time)
    .bind(&request.timezone)
    .bind(request.browser_notifications_enabled)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    get_or_create_preferences(pool, user_id).await
}

pub async fn list_due_reminders(pool: &DatabasePool, user_id: &str) -> Result<Vec<DueReminder>> {
    let rows = sqlx::query(
        "SELECT id, name, created_at, last_watered, last_fertilized, watering_interval_days, fertilizing_interval_days
         FROM plants
         WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let now = Utc::now();
    let mut due = Vec::new();

    for row in rows {
        let plant = PlantReminderRow {
            plant_id: row.try_get::<String, _>("id").map_err(AppError::Database)?,
            plant_name: row
                .try_get::<String, _>("name")
                .map_err(AppError::Database)?,
            created_at: row
                .try_get::<String, _>("created_at")
                .map_err(AppError::Database)?,
            last_watered: row
                .try_get::<Option<String>, _>("last_watered")
                .map_err(AppError::Database)?,
            last_fertilized: row
                .try_get::<Option<String>, _>("last_fertilized")
                .map_err(AppError::Database)?,
            watering_interval_days: row
                .try_get::<Option<i32>, _>("watering_interval_days")
                .map_err(AppError::Database)?,
            fertilizing_interval_days: row
                .try_get::<Option<i32>, _>("fertilizing_interval_days")
                .map_err(AppError::Database)?,
        };

        let created_at = parse_dt(&plant.created_at)?;

        if let Some(interval) = plant.watering_interval_days {
            let last = match &plant.last_watered {
                Some(v) => parse_dt(v)?,
                None => created_at,
            };
            let due_at = last + Duration::days(i64::from(interval));
            if due_at <= now {
                let due_date = due_at.format("%Y-%m-%d").to_string();
                let already_sent =
                    reminder_delivery_exists(pool, user_id, &plant.plant_id, "watering", &due_date)
                        .await?;
                due.push(DueReminder {
                    plant_id: plant.plant_id.clone(),
                    plant_name: plant.plant_name.clone(),
                    reminder_type: "watering".to_string(),
                    due_at: due_at.to_rfc3339(),
                    due_date,
                    days_overdue: (now - due_at).num_days().max(0),
                    already_sent,
                });
            }
        }

        if let Some(interval) = plant.fertilizing_interval_days {
            let last = match &plant.last_fertilized {
                Some(v) => parse_dt(v)?,
                None => created_at,
            };
            let due_at = last + Duration::days(i64::from(interval));
            if due_at <= now {
                let due_date = due_at.format("%Y-%m-%d").to_string();
                let already_sent = reminder_delivery_exists(
                    pool,
                    user_id,
                    &plant.plant_id,
                    "fertilizing",
                    &due_date,
                )
                .await?;
                due.push(DueReminder {
                    plant_id: plant.plant_id.clone(),
                    plant_name: plant.plant_name.clone(),
                    reminder_type: "fertilizing".to_string(),
                    due_at: due_at.to_rfc3339(),
                    due_date,
                    days_overdue: (now - due_at).num_days().max(0),
                    already_sent,
                });
            }
        }
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

        sqlx::query(
            "INSERT INTO reminder_deliveries
             (id, user_id, plant_id, reminder_type, due_date, due_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(user_id, plant_id, reminder_type, due_date)
             DO UPDATE SET updated_at = excluded.updated_at",
        )
        .bind(&id)
        .bind(user_id)
        .bind(&reminder.plant_id)
        .bind(&reminder.reminder_type)
        .bind(&reminder.due_date)
        .bind(&reminder.due_at)
        .bind(&now)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;

        let result = sqlx::query(
            "UPDATE reminder_deliveries
             SET sent_at = ?, updated_at = ?
             WHERE user_id = ?
               AND plant_id = ?
               AND reminder_type = ?
               AND due_date = ?
               AND sent_at IS NULL",
        )
        .bind(&now)
        .bind(&now)
        .bind(user_id)
        .bind(&reminder.plant_id)
        .bind(&reminder.reminder_type)
        .bind(&reminder.due_date)
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
    reminder_type: &str,
    due_date: &str,
) -> Result<bool> {
    let sent = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) as count
         FROM reminder_deliveries
         WHERE user_id = ?
           AND plant_id = ?
           AND reminder_type = ?
           AND due_date = ?
           AND sent_at IS NOT NULL",
    )
    .bind(user_id)
    .bind(plant_id)
    .bind(reminder_type)
    .bind(due_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(sent > 0)
}
