use chrono::{DateTime, Utc};
use sqlx::{FromRow, Row};
use uuid::Uuid;

use crate::database::care_tasks as db_care_tasks;
use crate::database::DatabasePool;
use crate::models::{
    CreatePlantRequest, CustomMetric, MetricDataType, PlantResponse, UpdatePlantRequest,
};
use crate::models::care_task::CreateCareTaskRequest;
use crate::utils::errors::AppError;

#[derive(Debug, FromRow)]
pub struct PlantRow {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub genus: String,
    pub preview_id: Option<String>,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl PlantRow {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_response(self) -> Result<PlantResponse, AppError> {
        Ok(PlantResponse {
            id: Uuid::parse_str(&self.id).map_err(|_| AppError::Internal {
                message: "Invalid UUID in database".to_string(),
            })?,
            name: self.name,
            genus: self.genus,
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
            care_tasks: vec![],
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

/// Enriches a PlantResponse with custom_metrics and care_tasks
async fn enrich_plant_response(
    pool: &DatabasePool,
    mut plant: PlantResponse,
) -> Result<PlantResponse, AppError> {
    let plant_id_str = plant.id.to_string();
    plant.custom_metrics = load_custom_metrics_for_plant(pool, &plant_id_str).await?;

    let care_response =
        db_care_tasks::list_care_tasks_for_plant(pool, &plant.id, &plant.user_id, false).await?;
    plant.care_tasks = care_response.tasks;

    Ok(plant)
}

pub async fn create_plant(
    pool: &DatabasePool,
    user_id: &str,
    request: &CreatePlantRequest,
) -> Result<PlantResponse, AppError> {
    let plant_id = Uuid::new_v4();
    let plant_id_str = plant_id.to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO plants (id, user_id, name, genus, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&plant_id_str)
    .bind(user_id)
    .bind(&request.name)
    .bind(&request.genus)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create plant: {}", e);
        AppError::Database(e)
    })?;

    // Create custom metrics if provided
    if let Some(custom_metrics) = &request.custom_metrics {
        let metrics = custom_metrics
            .iter()
            .map(|m| (None, m.name.clone(), m.unit.clone(), m.data_type.clone()))
            .collect::<Vec<_>>();
        replace_custom_metrics_for_plant(pool, &plant_id_str, &metrics).await?;
    }

    // Create care tasks if provided
    if let Some(care_tasks) = &request.care_tasks {
        for (i, ct) in care_tasks.iter().enumerate() {
            let req = CreateCareTaskRequest {
                name: ct.name.clone(),
                icon: ct.icon.clone(),
                color: ct.color.clone(),
                interval_days: ct.interval_days,
                amount: ct.amount,
                unit: ct.unit.clone(),
                notes: ct.notes.clone(),
                last_performed: ct.last_performed,
                sort_order: Some(i as i32),
            };
            db_care_tasks::create_care_task(pool, &plant_id, user_id, &req).await?;
        }
    }

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

    let plant = plant_row.map_or_else(
        || {
            Err(AppError::NotFound {
                resource: format!("Plant with id {plant_id}"),
            })
        },
        PlantRow::to_response,
    )?;

    enrich_plant_response(pool, plant).await
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
    let order_clause = match sort {
        Some("date_asc") => "ORDER BY created_at ASC",
        Some("name_asc") => "ORDER BY name ASC",
        Some("name_desc") => "ORDER BY name DESC",
        _ => "ORDER BY created_at DESC",
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

    let mut plants = Vec::new();
    for row in plant_rows {
        let plant = row.to_response()?;
        plants.push(enrich_plant_response(pool, plant).await?);
    }

    Ok((plants, total))
}

pub async fn update_plant(
    pool: &DatabasePool,
    plant_id: Uuid,
    user_id: &str,
    request: &UpdatePlantRequest,
) -> Result<PlantResponse, AppError> {
    let existing_plant = get_plant_by_id(pool, plant_id).await?;
    if existing_plant.user_id != user_id {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    let now = Utc::now().to_rfc3339();
    let plant_id_str = plant_id.to_string();

    let mut sets = vec!["updated_at = ?"];
    let mut binds: Vec<String> = vec![now.clone()];

    if let Some(name) = &request.name {
        sets.push("name = ?");
        binds.push(name.clone());
    }
    if let Some(genus) = &request.genus {
        sets.push("genus = ?");
        binds.push(genus.clone());
    }

    let query_str = format!(
        "UPDATE plants SET {} WHERE id = ? AND user_id = ?",
        sets.join(", ")
    );

    let mut query = sqlx::query(&query_str);
    for bind in &binds {
        query = query.bind(bind);
    }
    query = query.bind(&plant_id_str).bind(user_id);

    query.execute(pool).await.map_err(|e| {
        tracing::error!("Failed to update plant: {}", e);
        AppError::Database(e)
    })?;

    if let Some(custom_metrics) = &request.custom_metrics {
        let metrics = custom_metrics
            .iter()
            .map(|m| (m.id, m.name.clone(), m.unit.clone(), m.data_type.clone()))
            .collect::<Vec<_>>();
        replace_custom_metrics_for_plant(pool, &plant_id_str, &metrics).await?;
    }

    get_plant_by_id(pool, plant_id).await
}

pub async fn delete_plant(
    pool: &DatabasePool,
    plant_id: Uuid,
    user_id: &str,
) -> Result<(), AppError> {
    let plant_id_str = plant_id.to_string();

    let result = sqlx::query("DELETE FROM plants WHERE id = ? AND user_id = ?")
        .bind(&plant_id_str)
        .bind(user_id)
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

    let plant_exists = sqlx::query("SELECT 1 FROM plants WHERE id = ? AND user_id = ?")
        .bind(&plant_id_str)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

    if plant_exists.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    let photo_exists = sqlx::query("SELECT 1 FROM photos WHERE id = ? AND plant_id = ?")
        .bind(&photo_id_str)
        .bind(&plant_id_str)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

    if photo_exists.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Photo with id {photo_id} for plant {plant_id}"),
        });
    }

    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE plants SET preview_id = ?, updated_at = ? WHERE id = ? AND user_id = ?")
        .bind(&photo_id_str)
        .bind(&now)
        .bind(&plant_id_str)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

    get_plant_by_id(pool, plant_id).await
}

pub async fn clear_plant_preview(
    pool: &DatabasePool,
    plant_id: Uuid,
    user_id: &str,
) -> Result<PlantResponse, AppError> {
    let plant_id_str = plant_id.to_string();

    let plant_exists = sqlx::query("SELECT 1 FROM plants WHERE id = ? AND user_id = ?")
        .bind(&plant_id_str)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

    if plant_exists.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }

    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE plants SET preview_id = NULL, updated_at = ? WHERE id = ? AND user_id = ?",
    )
    .bind(&now)
    .bind(&plant_id_str)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    get_plant_by_id(pool, plant_id).await
}
