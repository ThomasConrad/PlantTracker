use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, Row};
use uuid::Uuid;

use crate::database::DatabasePool;
use crate::models::{
    CreatePlantRequest, CustomMetric, MetricDataType, PlantResponse, UpdatePlantRequest,
};
use crate::utils::errors::AppError;

#[derive(Debug, FromRow)]
pub struct PlantRow {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub genus: String,
    pub watering_interval_days: Option<i32>,
    pub fertilizing_interval_days: Option<i32>,
    pub watering_amount: Option<f64>,
    pub watering_unit: Option<String>,
    pub watering_notes: Option<String>,
    pub fertilizing_amount: Option<f64>,
    pub fertilizing_unit: Option<String>,
    pub fertilizing_notes: Option<String>,
    pub last_watered: Option<String>,
    pub last_fertilized: Option<String>,
    pub preview_id: Option<String>,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl PlantRow {
    /// Converts a `PlantRow` from the database into a `PlantResponse` for the API.
    ///
    /// # Errors
    ///
    /// Returns an error if the plant ID in the database is not a valid UUID.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_response(self) -> Result<PlantResponse, AppError> {
        Ok(PlantResponse {
            id: Uuid::parse_str(&self.id).map_err(|_| AppError::Internal {
                message: "Invalid UUID in database".to_string(),
            })?,
            name: self.name,
            genus: self.genus,
            watering_schedule: crate::models::plant::CareSchedule {
                interval_days: self.watering_interval_days,
                amount: self.watering_amount,
                unit: self.watering_unit,
                notes: self.watering_notes,
            },
            fertilizing_schedule: crate::models::plant::CareSchedule {
                interval_days: self.fertilizing_interval_days,
                amount: self.fertilizing_amount,
                unit: self.fertilizing_unit,
                notes: self.fertilizing_notes,
            },
            last_watered: self
                .last_watered
                .map(|s| s.parse::<DateTime<Utc>>())
                .transpose()
                .map_err(|_| AppError::Internal {
                    message: "Invalid datetime in database".to_string(),
                })?,
            last_fertilized: self
                .last_fertilized
                .map(|s| s.parse::<DateTime<Utc>>())
                .transpose()
                .map_err(|_| AppError::Internal {
                    message: "Invalid datetime in database".to_string(),
                })?,
            preview_id: self
                .preview_id
                .as_ref()
                .and_then(|s| Uuid::parse_str(s).ok()),
            preview_url: self
                .preview_id
                .as_ref()
                .map(|thumb_id| format!("/api/v1/plants/{}/photos/{}", self.id, thumb_id)),
            archived_at: self
                .archived_at
                .map(|s| s.parse::<DateTime<Utc>>())
                .transpose()
                .map_err(|_| AppError::Internal {
                    message: "Invalid datetime in database".to_string(),
                })?,
            custom_metrics: vec![],
            created_at: self.created_at.parse::<DateTime<Utc>>().map_err(|_| {
                AppError::Internal {
                    message: "Invalid datetime in database".to_string(),
                }
            })?,
            updated_at: self.updated_at.parse::<DateTime<Utc>>().map_err(|_| {
                AppError::Internal {
                    message: "Invalid datetime in database".to_string(),
                }
            })?,
            user_id: self.user_id,
        })
    }
}

fn metric_data_type_to_str(data_type: &MetricDataType) -> &'static str {
    match data_type {
        MetricDataType::Number => "number",
        MetricDataType::Text => "text",
        MetricDataType::Boolean => "boolean",
    }
}

fn parse_metric_data_type(value: &str) -> MetricDataType {
    match value.to_lowercase().as_str() {
        "number" => MetricDataType::Number,
        "text" => MetricDataType::Text,
        "boolean" => MetricDataType::Boolean,
        _ => MetricDataType::Text,
    }
}

async fn load_custom_metrics_for_plant(
    pool: &DatabasePool,
    plant_id: &str,
) -> Result<Vec<CustomMetric>, AppError> {
    let rows = sqlx::query(
        "SELECT id, plant_id, name, unit, data_type FROM custom_metrics WHERE plant_id = ? ORDER BY name ASC",
    )
    .bind(plant_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    rows.into_iter()
        .map(|row| {
            let id = row.try_get::<String, _>("id").map_err(AppError::Database)?;
            let plant_id = row
                .try_get::<String, _>("plant_id")
                .map_err(AppError::Database)?;
            let data_type = row
                .try_get::<String, _>("data_type")
                .map_err(AppError::Database)?;

            Ok(CustomMetric {
                id: Uuid::parse_str(&id).map_err(|_| AppError::Internal {
                    message: "Invalid custom metric id UUID in database".to_string(),
                })?,
                plant_id: Uuid::parse_str(&plant_id).map_err(|_| AppError::Internal {
                    message: "Invalid custom metric plant_id UUID in database".to_string(),
                })?,
                name: row
                    .try_get::<String, _>("name")
                    .map_err(AppError::Database)?,
                unit: row
                    .try_get::<String, _>("unit")
                    .map_err(AppError::Database)?,
                data_type: parse_metric_data_type(&data_type),
            })
        })
        .collect()
}

async fn replace_custom_metrics_for_plant(
    pool: &DatabasePool,
    plant_id: &str,
    metrics: &[(Option<Uuid>, String, String, MetricDataType)],
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM custom_metrics WHERE plant_id = ?")
        .bind(plant_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

    for (existing_id, name, unit, data_type) in metrics {
        let metric_id = existing_id.unwrap_or_else(Uuid::new_v4).to_string();
        sqlx::query(
            "INSERT INTO custom_metrics (id, plant_id, name, unit, data_type, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(metric_id)
        .bind(plant_id)
        .bind(name)
        .bind(unit)
        .bind(metric_data_type_to_str(data_type))
        .bind(Utc::now().to_rfc3339())
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    }

    Ok(())
}

/// Creates a new plant in the database for a specific user.
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `user_id` - ID of the user creating the plant
/// * `request` - Plant creation request data
///
/// # Errors
///
/// Returns an error if:
/// - Database insertion fails
/// - Invalid data provided
/// - User does not exist
pub async fn create_plant(
    pool: &DatabasePool,
    user_id: &str,
    request: &CreatePlantRequest,
) -> Result<PlantResponse, AppError> {
    let plant_id = Uuid::new_v4();
    let plant_id_str = plant_id.to_string();
    let now = Utc::now().to_rfc3339();

    // Extract values to avoid lifetime issues
    let watering_interval = request.watering_interval_days();
    let fertilizing_interval = request.fertilizing_interval_days();
    let watering_amount = request.watering_amount();
    let watering_unit = request.watering_unit();
    let watering_notes = request.watering_notes();
    let fertilizing_amount = request.fertilizing_amount();
    let fertilizing_unit = request.fertilizing_unit();
    let fertilizing_notes = request.fertilizing_notes();
    let last_watered = request.last_watered.map(|dt| dt.to_rfc3339());
    let last_fertilized = request.last_fertilized.map(|dt| dt.to_rfc3339());

    let result = sqlx::query!(
        r#"
        INSERT INTO plants (
            id, user_id, name, genus, 
            watering_interval_days, fertilizing_interval_days,
            watering_amount, watering_unit, watering_notes,
            fertilizing_amount, fertilizing_unit, fertilizing_notes,
            last_watered, last_fertilized,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        plant_id_str,
        user_id,
        request.name,
        request.genus,
        watering_interval,
        fertilizing_interval,
        watering_amount,
        watering_unit,
        watering_notes,
        fertilizing_amount,
        fertilizing_unit,
        fertilizing_notes,
        last_watered,
        last_fertilized,
        now,
        now
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create plant: {}", e);
        AppError::Database(e)
    })?;

    if result.rows_affected() != 1 {
        return Err(AppError::Internal {
            message: "Failed to create plant".to_string(),
        });
    }

    if let Some(custom_metrics) = &request.custom_metrics {
        let metrics = custom_metrics
            .iter()
            .map(|m| (None, m.name.clone(), m.unit.clone(), m.data_type.clone()))
            .collect::<Vec<_>>();
        replace_custom_metrics_for_plant(pool, &plant_id_str, &metrics).await?;
    }

    // Return the created plant
    get_plant_by_id(pool, plant_id).await
}

pub async fn get_plant_by_id(
    pool: &DatabasePool,
    plant_id: Uuid,
) -> Result<PlantResponse, AppError> {
    let plant_id_str = plant_id.to_string();
    let plant_row = sqlx::query_as::<_, PlantRow>("SELECT * FROM plants WHERE id = ?")
        .bind(plant_id_str)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch plant: {}", e);
            AppError::Database(e)
        })?;

    let mut plant = plant_row.map_or_else(
        || {
            Err(AppError::NotFound {
                resource: format!("Plant with id {plant_id}"),
            })
        },
        PlantRow::to_response,
    )?;

    plant.custom_metrics = load_custom_metrics_for_plant(pool, &plant.id.to_string()).await?;
    Ok(plant)
}

pub async fn list_plants_for_user(
    pool: &DatabasePool,
    user_id: &str,
    limit: i64,
    offset: i64,
    search: Option<&str>,
) -> Result<(Vec<PlantResponse>, i64), AppError> {
    list_plants_for_user_with_sort(pool, user_id, limit, offset, search, None, false).await
}

pub async fn list_plants_for_user_with_sort(
    pool: &DatabasePool,
    user_id: &str,
    limit: i64,
    offset: i64,
    search: Option<&str>,
    sort: Option<&str>,
    include_archived: bool,
) -> Result<(Vec<PlantResponse>, i64), AppError> {
    // Determine sort order
    let order_clause = match sort {
        Some("date_asc") => "ORDER BY created_at ASC",
        Some("name_asc") => "ORDER BY name ASC",
        Some("name_desc") => "ORDER BY name DESC",
        _ => "ORDER BY created_at DESC", // default
    };

    let archived_clause = if include_archived {
        ""
    } else {
        " AND archived_at IS NULL"
    };

    let (query, count_query, search_param) = search.map_or((
            format!("SELECT * FROM plants WHERE user_id = ? {} {} LIMIT ? OFFSET ?", archived_clause, order_clause),
            format!("SELECT COUNT(*) as count FROM plants WHERE user_id = ?{}", archived_clause),
            None
        ), |search_term| {
        let search_pattern = format!("%{search_term}%");
        (
            format!("SELECT * FROM plants WHERE user_id = ? AND (name LIKE ? OR genus LIKE ?) {} {} LIMIT ? OFFSET ?", archived_clause, order_clause),
            format!("SELECT COUNT(*) as count FROM plants WHERE user_id = ? AND (name LIKE ? OR genus LIKE ?){}", archived_clause),
            Some(search_pattern)
        )
    });

    // Get total count
    let total = if let Some(search_param) = &search_param {
        sqlx::query(&count_query)
            .bind(user_id)
            .bind(search_param)
            .bind(search_param)
            .fetch_one(pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to count plants: {}", e);
                AppError::Database(e)
            })?
            .get::<i64, _>("count")
    } else {
        sqlx::query(&count_query)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to count plants: {}", e);
                AppError::Database(e)
            })?
            .get::<i64, _>("count")
    };

    // Get plants
    let plant_rows = if let Some(search_param) = &search_param {
        sqlx::query_as::<_, PlantRow>(&query)
            .bind(user_id)
            .bind(search_param)
            .bind(search_param)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    } else {
        sqlx::query_as::<_, PlantRow>(&query)
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    }
    .map_err(|e| {
        tracing::error!("Failed to fetch plants: {}", e);
        AppError::Database(e)
    })?;

    let plants = plant_rows
        .into_iter()
        .map(PlantRow::to_response)
        .collect::<Result<Vec<_>, _>>()?;

    let mut plants = plants;
    for plant in &mut plants {
        plant.custom_metrics = load_custom_metrics_for_plant(pool, &plant.id.to_string()).await?;
    }

    Ok((plants, total))
}

pub async fn update_plant(
    pool: &DatabasePool,
    plant_id: Uuid,
    user_id: &str,
    request: &UpdatePlantRequest,
) -> Result<PlantResponse, AppError> {
    // First verify the plant exists and belongs to the user
    let existing_plant = get_plant_by_id(pool, plant_id).await?;
    if existing_plant.user_id != user_id {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    let now = Utc::now().to_rfc3339();

    // Build the UPDATE query with proper parameter handling
    let query = "
        UPDATE plants SET 
            name = COALESCE(?, name),
            genus = COALESCE(?, genus),
            watering_interval_days = CASE WHEN ? THEN ? WHEN ? THEN NULL ELSE watering_interval_days END,
            fertilizing_interval_days = CASE WHEN ? THEN ? WHEN ? THEN NULL ELSE fertilizing_interval_days END,
            watering_amount = CASE WHEN ? THEN ? WHEN ? THEN NULL ELSE watering_amount END,
            watering_unit = CASE WHEN ? THEN ? WHEN ? THEN NULL ELSE watering_unit END,
            watering_notes = CASE WHEN ? THEN ? WHEN ? THEN NULL ELSE watering_notes END,
            fertilizing_amount = CASE WHEN ? THEN ? WHEN ? THEN NULL ELSE fertilizing_amount END,
            fertilizing_unit = CASE WHEN ? THEN ? WHEN ? THEN NULL ELSE fertilizing_unit END,
            fertilizing_notes = CASE WHEN ? THEN ? WHEN ? THEN NULL ELSE fertilizing_notes END,
            updated_at = ?
        WHERE id = ? AND user_id = ?
    ";

    let mut query_builder = sqlx::query(query).bind(&request.name).bind(&request.genus);

    // Handle watering schedule fields with explicit null handling
    let watering_schedule_provided = request.watering_schedule.is_some();

    // Watering interval days
    if let Some(watering_interval) = request.watering_interval_days() {
        query_builder = query_builder.bind(true).bind(watering_interval).bind(false);
    } else if watering_schedule_provided {
        // Schedule provided but interval is None = explicitly disabled
        query_builder = query_builder
            .bind(false)
            .bind(None::<Option<i32>>)
            .bind(true);
    } else {
        // Schedule not provided = no change
        query_builder = query_builder
            .bind(false)
            .bind(None::<Option<i32>>)
            .bind(false);
    }

    // Fertilizing interval days
    let fertilizing_schedule_provided = request.fertilizing_schedule.is_some();
    if let Some(fertilizing_interval) = request.fertilizing_interval_days() {
        query_builder = query_builder
            .bind(true)
            .bind(fertilizing_interval)
            .bind(false);
    } else if fertilizing_schedule_provided {
        // Schedule provided but interval is None = explicitly disabled
        query_builder = query_builder
            .bind(false)
            .bind(None::<Option<i32>>)
            .bind(true);
    } else {
        // Schedule not provided = no change
        query_builder = query_builder
            .bind(false)
            .bind(None::<Option<i32>>)
            .bind(false);
    }

    // Watering amount
    if let Some(watering_amount) = request.watering_amount() {
        query_builder = query_builder.bind(true).bind(watering_amount).bind(false);
    } else if watering_schedule_provided {
        query_builder = query_builder
            .bind(false)
            .bind(None::<Option<f64>>)
            .bind(true);
    } else {
        query_builder = query_builder
            .bind(false)
            .bind(None::<Option<f64>>)
            .bind(false);
    }

    // Watering unit
    if let Some(watering_unit) = request.watering_unit() {
        query_builder = query_builder.bind(true).bind(watering_unit).bind(false);
    } else if watering_schedule_provided {
        query_builder = query_builder
            .bind(false)
            .bind(None::<Option<String>>)
            .bind(true);
    } else {
        query_builder = query_builder
            .bind(false)
            .bind(None::<Option<String>>)
            .bind(false);
    }

    // Watering notes
    if let Some(watering_notes) = request.watering_notes() {
        query_builder = query_builder.bind(true).bind(watering_notes).bind(false);
    } else if watering_schedule_provided {
        query_builder = query_builder
            .bind(false)
            .bind(None::<Option<String>>)
            .bind(true);
    } else {
        query_builder = query_builder
            .bind(false)
            .bind(None::<Option<String>>)
            .bind(false);
    }

    // Fertilizing amount
    if let Some(fertilizing_amount) = request.fertilizing_amount() {
        query_builder = query_builder
            .bind(true)
            .bind(fertilizing_amount)
            .bind(false);
    } else if fertilizing_schedule_provided {
        query_builder = query_builder
            .bind(false)
            .bind(None::<Option<f64>>)
            .bind(true);
    } else {
        query_builder = query_builder
            .bind(false)
            .bind(None::<Option<f64>>)
            .bind(false);
    }

    // Fertilizing unit
    if let Some(fertilizing_unit) = request.fertilizing_unit() {
        query_builder = query_builder.bind(true).bind(fertilizing_unit).bind(false);
    } else if fertilizing_schedule_provided {
        query_builder = query_builder
            .bind(false)
            .bind(None::<Option<String>>)
            .bind(true);
    } else {
        query_builder = query_builder
            .bind(false)
            .bind(None::<Option<String>>)
            .bind(false);
    }

    // Fertilizing notes
    if let Some(fertilizing_notes) = request.fertilizing_notes() {
        query_builder = query_builder.bind(true).bind(fertilizing_notes).bind(false);
    } else if fertilizing_schedule_provided {
        query_builder = query_builder
            .bind(false)
            .bind(None::<Option<String>>)
            .bind(true);
    } else {
        query_builder = query_builder
            .bind(false)
            .bind(None::<Option<String>>)
            .bind(false);
    }

    query_builder = query_builder
        .bind(&now)
        .bind(plant_id.to_string())
        .bind(user_id);

    let result = query_builder.execute(pool).await.map_err(|e| {
        tracing::error!("Failed to update plant: {}", e);
        AppError::Database(e)
    })?;

    if result.rows_affected() != 1 {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    if let Some(custom_metrics) = &request.custom_metrics {
        let metrics = custom_metrics
            .iter()
            .map(|m| (m.id, m.name.clone(), m.unit.clone(), m.data_type.clone()))
            .collect::<Vec<_>>();
        replace_custom_metrics_for_plant(pool, &plant_id.to_string(), &metrics).await?;
    }

    // Return the updated plant
    get_plant_by_id(pool, plant_id).await
}

pub async fn delete_plant(
    pool: &DatabasePool,
    plant_id: Uuid,
    user_id: &str,
) -> Result<(), AppError> {
    let plant_id_str = plant_id.to_string();

    let result = sqlx::query!(
        "DELETE FROM plants WHERE id = ? AND user_id = ?",
        plant_id_str,
        user_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete plant: {}", e);
        AppError::Database(e)
    })?;

    if result.rows_affected() != 1 {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    Ok(())
}

pub async fn archive_plant(
    pool: &DatabasePool,
    plant_id: Uuid,
    user_id: &str,
) -> Result<PlantResponse, AppError> {
    let plant_id_str = plant_id.to_string();
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
        "UPDATE plants SET archived_at = ?, updated_at = ? WHERE id = ? AND user_id = ?",
    )
    .bind(&now)
    .bind(&now)
    .bind(&plant_id_str)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to archive plant: {}", e);
        AppError::Database(e)
    })?;

    if result.rows_affected() != 1 {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    get_plant_by_id(pool, plant_id).await
}

pub async fn unarchive_plant(
    pool: &DatabasePool,
    plant_id: Uuid,
    user_id: &str,
) -> Result<PlantResponse, AppError> {
    let plant_id_str = plant_id.to_string();
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
        "UPDATE plants SET archived_at = NULL, updated_at = ? WHERE id = ? AND user_id = ?",
    )
    .bind(&now)
    .bind(&plant_id_str)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to unarchive plant: {}", e);
        AppError::Database(e)
    })?;

    if result.rows_affected() != 1 {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    get_plant_by_id(pool, plant_id).await
}

pub async fn set_plant_preview(
    pool: &DatabasePool,
    plant_id: Uuid,
    photo_id: Uuid,
    user_id: &str,
) -> Result<PlantResponse, AppError> {
    let plant_id_str = plant_id.to_string();
    let photo_id_str = photo_id.to_string();

    // First verify the plant exists and belongs to the user
    let plant_exists = sqlx::query("SELECT 1 FROM plants WHERE id = ? AND user_id = ?")
        .bind(&plant_id_str)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check plant existence: {}", e);
            AppError::Database(e)
        })?;

    if plant_exists.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    // Verify the photo exists and belongs to the plant
    let photo_exists = sqlx::query("SELECT 1 FROM photos WHERE id = ? AND plant_id = ?")
        .bind(&photo_id_str)
        .bind(&plant_id_str)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check photo existence: {}", e);
            AppError::Database(e)
        })?;

    if photo_exists.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Photo with id {photo_id} for plant {plant_id}"),
        });
    }

    // Update the plant's preview_id
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query!(
        "UPDATE plants SET preview_id = ?, updated_at = ? WHERE id = ? AND user_id = ?",
        photo_id_str,
        now,
        plant_id_str,
        user_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update plant thumbnail: {}", e);
        AppError::Database(e)
    })?;

    if result.rows_affected() != 1 {
        return Err(AppError::Internal {
            message: "Failed to update plant thumbnail".to_string(),
        });
    }

    // Return the updated plant
    get_plant_by_id(pool, plant_id).await
}
pub async fn clear_plant_preview(
    pool: &DatabasePool,
    plant_id: Uuid,
    user_id: &str,
) -> Result<PlantResponse, AppError> {
    let plant_id_str = plant_id.to_string();

    // First verify the plant exists and belongs to the user
    let plant_exists = sqlx::query("SELECT 1 FROM plants WHERE id = ? AND user_id = ?")
        .bind(&plant_id_str)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    if plant_exists.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    // Clear the plant's preview_id
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query!(
        "UPDATE plants SET preview_id = NULL, updated_at = ? WHERE id = ? AND user_id = ?",
        now,
        plant_id_str,
        user_id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    // Return the updated plant
    get_plant_by_id(pool, plant_id).await
}
