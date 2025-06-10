#[allow(unused_imports)]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use utoipa::OpenApi;
use uuid::Uuid;

use crate::auth::AuthSession;
use crate::database::{tracking as db_tracking, DatabasePool};
use crate::middleware::validation::ValidatedJson;
use crate::models::tracking_entry::{
    CreateTrackingEntryRequest, TrackingEntriesResponse, TrackingEntry,
};
use crate::utils::errors::{AppError, Result};

pub fn routes() -> Router<DatabasePool> {
    Router::new()
        .route("/:plant_id/entries", get(list_entries).post(create_entry))
        .route(
            "/:plant_id/entries/:entry_id",
            get(get_entry).put(update_entry).delete(delete_entry),
        )
}

#[utoipa::path(
    get,
    path = "/plants/{plant_id}/entries",
    responses(
        (status = 200, description = "List tracking entries for plant", body = TrackingEntriesResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Plant not found"),
    ),
    params(
        ("plant_id" = Uuid, Path, description = "Plant ID")
    ),
    security(
        ("session" = [])
    )
)]
async fn list_entries(
    auth_session: AuthSession,
    State(pool): State<DatabasePool>,
    Path(plant_id): Path<Uuid>,
) -> Result<Json<TrackingEntriesResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!(
        "List tracking entries request for plant: {} by user: {}",
        plant_id,
        user.id
    );

    let response = db_tracking::get_tracking_entries_for_plant(&pool, &plant_id, &user.id).await?;

    tracing::debug!(
        "Returning {} tracking entries for plant: {}",
        response.total,
        plant_id
    );
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/plants/{plant_id}/entries",
    request_body = CreateTrackingEntryRequest,
    responses(
        (status = 201, description = "Tracking entry created", body = TrackingEntry),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Plant not found"),
    ),
    params(
        ("plant_id" = Uuid, Path, description = "Plant ID")
    ),
    security(
        ("session" = [])
    )
)]
async fn create_entry(
    auth_session: AuthSession,
    State(pool): State<DatabasePool>,
    Path(plant_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<CreateTrackingEntryRequest>,
) -> Result<(StatusCode, Json<TrackingEntry>)> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!(
        "Create tracking entry request for plant: {} by user: {}",
        plant_id,
        user.id
    );

    let entry = db_tracking::create_tracking_entry(&pool, &plant_id, &user.id, &payload).await?;

    tracing::info!(
        "Created tracking entry with id: {} for plant: {}",
        entry.id,
        plant_id
    );
    Ok((StatusCode::CREATED, Json(entry)))
}

async fn get_entry(
    auth_session: AuthSession,
    State(pool): State<DatabasePool>,
    Path((plant_id, entry_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<TrackingEntry>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!(
        "Get tracking entry request for plant: {}, entry: {} by user: {}",
        plant_id,
        entry_id,
        user.id
    );

    let entry = db_tracking::get_tracking_entry(&pool, &plant_id, &entry_id, &user.id).await?;

    tracing::debug!(
        "Retrieved tracking entry: {} for plant: {}",
        entry_id,
        plant_id
    );
    Ok(Json(entry))
}

async fn update_entry(
    auth_session: AuthSession,
    State(pool): State<DatabasePool>,
    Path((plant_id, entry_id)): Path<(Uuid, Uuid)>,
    ValidatedJson(payload): ValidatedJson<crate::models::tracking_entry::UpdateTrackingEntryRequest>,
) -> Result<Json<TrackingEntry>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!(
        "Update tracking entry request for plant: {}, entry: {} by user: {}",
        plant_id,
        entry_id,
        user.id
    );

    let entry = db_tracking::update_tracking_entry(&pool, &plant_id, &entry_id, &user.id, &payload).await?;

    tracing::info!(
        "Updated tracking entry: {} for plant: {}",
        entry_id,
        plant_id
    );
    Ok(Json(entry))
}

async fn delete_entry(
    auth_session: AuthSession,
    State(pool): State<DatabasePool>,
    Path((plant_id, entry_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!(
        "Delete tracking entry request for plant: {}, entry: {} by user: {}",
        plant_id,
        entry_id,
        user.id
    );

    db_tracking::delete_tracking_entry(&pool, &plant_id, &entry_id, &user.id).await?;

    tracing::info!(
        "Deleted tracking entry: {} for plant: {}",
        entry_id,
        plant_id
    );
    Ok(StatusCode::NO_CONTENT)
}
