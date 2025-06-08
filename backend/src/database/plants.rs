use sqlx::{Row, FromRow};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;

use crate::database::DatabasePool;
use crate::models::{CreatePlantRequest, PlantResponse, UpdatePlantRequest};
use crate::utils::errors::AppError;

#[derive(Debug, FromRow)]
pub struct PlantRow {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub genus: String,
    pub watering_interval_days: i32,
    pub fertilizing_interval_days: i32,
    pub last_watered: Option<String>,
    pub last_fertilized: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl PlantRow {
    pub fn to_response(self) -> Result<PlantResponse, AppError> {
        Ok(PlantResponse {
            id: Uuid::parse_str(&self.id).map_err(|_| AppError::Internal { 
                message: "Invalid UUID in database".to_string() 
            })?,
            name: self.name,
            genus: self.genus,
            watering_interval_days: self.watering_interval_days,
            fertilizing_interval_days: self.fertilizing_interval_days,
            last_watered: self.last_watered
                .map(|s| s.parse::<DateTime<Utc>>())
                .transpose()
                .map_err(|_| AppError::Internal { 
                    message: "Invalid datetime in database".to_string() 
                })?,
            last_fertilized: self.last_fertilized
                .map(|s| s.parse::<DateTime<Utc>>())
                .transpose()
                .map_err(|_| AppError::Internal { 
                    message: "Invalid datetime in database".to_string() 
                })?,
            custom_metrics: vec![], // TODO: Load custom metrics
            created_at: self.created_at.parse::<DateTime<Utc>>().map_err(|_| AppError::Internal { 
                message: "Invalid datetime in database".to_string() 
            })?,
            updated_at: self.updated_at.parse::<DateTime<Utc>>().map_err(|_| AppError::Internal { 
                message: "Invalid datetime in database".to_string() 
            })?,
            user_id: self.user_id,
        })
    }
}

pub async fn create_plant(
    pool: &DatabasePool,
    user_id: &str,
    request: &CreatePlantRequest,
) -> Result<PlantResponse, AppError> {
    let plant_id = Uuid::new_v4();
    let plant_id_str = plant_id.to_string();
    let now = Utc::now().to_rfc3339();
    
    let result = sqlx::query!(
        r#"
        INSERT INTO plants (
            id, user_id, name, genus, 
            watering_interval_days, fertilizing_interval_days,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        plant_id_str,
        user_id,
        request.name,
        request.genus,
        request.watering_interval_days,
        request.fertilizing_interval_days,
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

    // Return the created plant
    get_plant_by_id(pool, plant_id).await
}

pub async fn get_plant_by_id(pool: &DatabasePool, plant_id: Uuid) -> Result<PlantResponse, AppError> {
    let plant_id_str = plant_id.to_string();
    let plant_row = sqlx::query_as::<_, PlantRow>(
        "SELECT * FROM plants WHERE id = ?"
    )
    .bind(plant_id_str)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch plant: {}", e);
        AppError::Database(e)
    })?;

    match plant_row {
        Some(plant) => plant.to_response(),
        None => Err(AppError::NotFound {
            resource: format!("Plant with id {}", plant_id),
        }),
    }
}

pub async fn list_plants_for_user(
    pool: &DatabasePool,
    user_id: &str,
    limit: i64,
    offset: i64,
    search: Option<&str>,
) -> Result<(Vec<PlantResponse>, i64), AppError> {
    let (query, count_query, search_param) = if let Some(search_term) = search {
        let search_pattern = format!("%{}%", search_term);
        (
            "SELECT * FROM plants WHERE user_id = ? AND (name LIKE ? OR genus LIKE ?) ORDER BY created_at DESC LIMIT ? OFFSET ?",
            "SELECT COUNT(*) as count FROM plants WHERE user_id = ? AND (name LIKE ? OR genus LIKE ?)",
            Some(search_pattern)
        )
    } else {
        (
            "SELECT * FROM plants WHERE user_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
            "SELECT COUNT(*) as count FROM plants WHERE user_id = ?",
            None
        )
    };

    // Get total count
    let total = if let Some(search_param) = &search_param {
        sqlx::query(count_query)
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
        sqlx::query(count_query)
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
        sqlx::query_as::<_, PlantRow>(query)
            .bind(user_id)
            .bind(search_param)
            .bind(search_param)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    } else {
        sqlx::query_as::<_, PlantRow>(query)
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
        .map(|row| row.to_response())
        .collect::<Result<Vec<_>, _>>()?;

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
            resource: format!("Plant with id {}", plant_id),
        });
    }

    let now = Utc::now().to_rfc3339();
    
    // Build dynamic update query based on provided fields
    let mut set_clauses = vec!["updated_at = ?"];
    let mut params: Vec<String> = vec![now.clone()];
    
    if let Some(name) = &request.name {
        set_clauses.push("name = ?");
        params.push(name.clone());
    }
    
    if let Some(genus) = &request.genus {
        set_clauses.push("genus = ?");
        params.push(genus.clone());
    }
    
    if let Some(watering_interval) = request.watering_interval_days {
        set_clauses.push("watering_interval_days = ?");
        params.push(watering_interval.to_string());
    }
    
    if let Some(fertilizing_interval) = request.fertilizing_interval_days {
        set_clauses.push("fertilizing_interval_days = ?");
        params.push(fertilizing_interval.to_string());
    }

    let query = format!(
        "UPDATE plants SET {} WHERE id = ? AND user_id = ?",
        set_clauses.join(", ")
    );
    
    params.push(plant_id.to_string());
    params.push(user_id.to_string());

    let mut query_builder = sqlx::query(&query);
    for param in params {
        query_builder = query_builder.bind(param);
    }

    let result = query_builder
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update plant: {}", e);
            AppError::Database(e)
        })?;

    if result.rows_affected() != 1 {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {}", plant_id),
        });
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
            resource: format!("Plant with id {}", plant_id),
        });
    }

    Ok(())
}