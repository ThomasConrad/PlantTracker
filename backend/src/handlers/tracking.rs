#[allow(unused_imports)]
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::AuthSession;
use crate::database::tracking as db_tracking;
use crate::middleware::validation::ValidatedJson;
use crate::models::tracking_entry::{
    CreateTrackingEntryRequest, TrackingEntriesResponse, TrackingEntry,
};
use crate::utils::errors::{AppError, Result};

#[derive(Debug, Deserialize)]
struct ListEntriesQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    sort: Option<String>,       // "date_asc", "date_desc" (default)
    entry_type: Option<String>, // filter by entry type
}

pub fn routes() -> Router<AppState> {
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
    State(app_state): State<AppState>,
    Path(plant_id): Path<Uuid>,
    Query(params): Query<ListEntriesQuery>,
) -> Result<Json<TrackingEntriesResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!(
        "List tracking entries request for plant: {} by user: {} with params: {:?}",
        plant_id,
        user.id,
        params
    );

    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    let sort_desc = match params.sort.as_deref() {
        Some("date_asc") => false,
        _ => true, // default to date_desc
    };

    let response = db_tracking::get_tracking_entries_for_plant_paginated(
        &app_state.pool,
        &plant_id,
        &user.id,
        limit,
        offset,
        sort_desc,
        params.entry_type.as_deref(),
    )
    .await?;

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
    State(app_state): State<AppState>,
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

    let entry = db_tracking::create_tracking_entry(&app_state.pool, &plant_id, &user.id, &payload).await?;

    tracing::info!(
        "Created tracking entry with id: {} for plant: {}",
        entry.id,
        plant_id
    );
    Ok((StatusCode::CREATED, Json(entry)))
}

async fn get_entry(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
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

    let entry = db_tracking::get_tracking_entry(&app_state.pool, &plant_id, &entry_id, &user.id).await?;

    tracing::debug!(
        "Retrieved tracking entry: {} for plant: {}",
        entry_id,
        plant_id
    );
    Ok(Json(entry))
}

async fn update_entry(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path((plant_id, entry_id)): Path<(Uuid, Uuid)>,
    ValidatedJson(payload): ValidatedJson<
        crate::models::tracking_entry::UpdateTrackingEntryRequest,
    >,
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

    let entry =
        db_tracking::update_tracking_entry(&app_state.pool, &plant_id, &entry_id, &user.id, &payload).await?;

    tracing::info!(
        "Updated tracking entry: {} for plant: {}",
        entry_id,
        plant_id
    );
    Ok(Json(entry))
}

async fn delete_entry(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
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

    db_tracking::delete_tracking_entry(&app_state.pool, &plant_id, &entry_id, &user.id).await?;

    tracing::info!(
        "Deleted tracking entry: {} for plant: {}",
        entry_id,
        plant_id
    );
    Ok(StatusCode::NO_CONTENT)
}
