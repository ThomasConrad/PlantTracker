use chrono::Utc;
use uuid::Uuid;

use crate::database::DatabasePool;
use crate::models::plant_attribute::{
    AttributeSource, CreatePlantAttributeRequest, PlantAttribute, PlantAttributeRow,
    PlantAttributesResponse,
};
use crate::utils::errors::AppError;

// ─── List ────────────────────────────────────────────────────────────────────

pub async fn list_attributes_for_plant(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
) -> Result<PlantAttributesResponse, AppError> {
    let plant_id_str = plant_id.to_string();
    let rows = sqlx::query_as!(
        PlantAttributeRow,
        r#"SELECT id, plant_id, user_id, key, label, value, icon as "icon?: String", category as "category?: String", source, sort_order, created_at, updated_at
         FROM plant_attributes WHERE plant_id = ? AND user_id = ? ORDER BY sort_order, label"#,
        plant_id_str,
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let attributes = rows.into_iter().map(|r| r.into_response()).collect();
    Ok(PlantAttributesResponse { attributes })
}

// ─── Get ─────────────────────────────────────────────────────────────────────

pub async fn get_attribute(
    pool: &DatabasePool,
    attribute_id: &Uuid,
    user_id: &str,
) -> Result<PlantAttribute, AppError> {
    let id_str = attribute_id.to_string();
    let row = sqlx::query_as!(
        PlantAttributeRow,
        r#"SELECT id, plant_id, user_id, key, label, value, icon as "icon?: String", category as "category?: String", source, sort_order, created_at, updated_at
         FROM plant_attributes WHERE id = ? AND user_id = ?"#,
        id_str,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?
    .ok_or(AppError::NotFound {
        resource: format!("Attribute {attribute_id}"),
    })?;

    Ok(row.into_response())
}

// ─── Create (upsert by key) ──────────────────────────────────────────────────

pub async fn upsert_attribute(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
    request: &CreatePlantAttributeRequest,
) -> Result<PlantAttribute, AppError> {
    let now = Utc::now().to_rfc3339();
    let plant_id_str = plant_id.to_string();
    let source_str = request
        .source
        .as_ref()
        .unwrap_or(&AttributeSource::User)
        .to_string();
    let sort_order = request.sort_order.unwrap_or(0);

    // Check if attribute with this key already exists for this plant
    let existing = sqlx::query_as!(
        PlantAttributeRow,
        r#"SELECT id, plant_id, user_id, key, label, value, icon as "icon?: String", category as "category?: String", source, sort_order, created_at, updated_at
         FROM plant_attributes WHERE plant_id = ? AND user_id = ? AND key = ?"#,
        plant_id_str,
        user_id,
        request.key
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    if let Some(existing_row) = existing {
        // Update existing attribute
        let existing_id = &existing_row.id;
        sqlx::query!(
            r#"UPDATE plant_attributes SET label = ?, value = ?, icon = ?, category = ?, source = ?, sort_order = ?, updated_at = ?
             WHERE id = ?"#,
            request.label,
            request.value,
            request.icon,
            request.category,
            source_str,
            sort_order,
            now,
            existing_id
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        let id = existing_row.id.parse().unwrap_or_default();
        get_attribute(pool, &id, user_id).await
    } else {
        // Create new attribute
        let id = Uuid::new_v4();
        let id_str = id.to_string();

        sqlx::query!(
            r#"INSERT INTO plant_attributes (id, plant_id, user_id, key, label, value, icon, category, source, sort_order, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            id_str,
            plant_id_str,
            user_id,
            request.key,
            request.label,
            request.value,
            request.icon,
            request.category,
            source_str,
            sort_order,
            now,
            now
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        get_attribute(pool, &id, user_id).await
    }
}

// ─── Update ──────────────────────────────────────────────────────────────────

pub async fn update_attribute(
    pool: &DatabasePool,
    attribute_id: &Uuid,
    user_id: &str,
    label: Option<&str>,
    value: Option<&str>,
    icon: Option<Option<&str>>,
    category: Option<Option<&str>>,
) -> Result<PlantAttribute, AppError> {
    // Verify exists and belongs to user
    let _existing = get_attribute(pool, attribute_id, user_id).await?;

    let now = Utc::now().to_rfc3339();
    let id_str = attribute_id.to_string();

    if let Some(label) = label {
        sqlx::query!(
            r#"UPDATE plant_attributes SET label = ?, updated_at = ? WHERE id = ? AND user_id = ?"#,
            label,
            now,
            id_str,
            user_id
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    }

    if let Some(value) = value {
        sqlx::query!(
            r#"UPDATE plant_attributes SET value = ?, updated_at = ? WHERE id = ? AND user_id = ?"#,
            value,
            now,
            id_str,
            user_id
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    }

    if let Some(icon) = icon {
        sqlx::query!(
            r#"UPDATE plant_attributes SET icon = ?, updated_at = ? WHERE id = ? AND user_id = ?"#,
            icon,
            now,
            id_str,
            user_id
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    }

    if let Some(category) = category {
        sqlx::query!(
            r#"UPDATE plant_attributes SET category = ?, updated_at = ? WHERE id = ? AND user_id = ?"#,
            category,
            now,
            id_str,
            user_id
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    }

    get_attribute(pool, attribute_id, user_id).await
}

// ─── Delete ──────────────────────────────────────────────────────────────────

pub async fn delete_attribute(
    pool: &DatabasePool,
    attribute_id: &Uuid,
    user_id: &str,
) -> Result<(), AppError> {
    let id_str = attribute_id.to_string();
    let result = sqlx::query!(
        r#"DELETE FROM plant_attributes WHERE id = ? AND user_id = ?"#,
        id_str,
        user_id
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound {
            resource: format!("Attribute {attribute_id}"),
        });
    }

    Ok(())
}

// ─── Bulk create (for plant creation / identification) ───────────────────────

pub async fn bulk_create_attributes(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
    attributes: &[CreatePlantAttributeRequest],
) -> Result<PlantAttributesResponse, AppError> {
    for (i, attr) in attributes.iter().enumerate() {
        let mut attr_with_order = attr.clone();
        if attr_with_order.sort_order.is_none() {
            attr_with_order.sort_order = Some(i as i32);
        }
        upsert_attribute(pool, plant_id, user_id, &attr_with_order).await?;
    }

    list_attributes_for_plant(pool, plant_id, user_id).await
}

// ─── Get context for coach ───────────────────────────────────────────────────

pub async fn get_attributes_context(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
) -> Result<String, AppError> {
    let response = list_attributes_for_plant(pool, plant_id, user_id).await?;

    if response.attributes.is_empty() {
        return Ok(String::new());
    }

    let mut context = String::from("\n## Plant Requirements & Characteristics\n");
    for attr in &response.attributes {
        let icon = attr.icon.as_deref().unwrap_or("");
        context.push_str(&format!("- {}{}: {}\n", icon, attr.label, attr.value));
    }
    context.push('\n');

    Ok(context)
}
