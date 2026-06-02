use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::AuthSession;
use crate::database::care_tasks as db;
use crate::middleware::validation::ValidatedJson;
use crate::models::care_task::{
    CareTaskWithStatus, CareTasksResponse, CreateCareTaskRequest, LogCareTaskRequest,
    ReorderCareTasksRequest, UpdateCareTaskRequest,
};
use crate::models::tracking_entry::TrackingEntry;
use crate::utils::errors::{AppError, Result};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/:plant_id/care-tasks",
            get(list_care_tasks).post(create_care_task),
        )
        .route("/:plant_id/care-tasks/reorder", put(reorder_care_tasks))
        .route(
            "/:plant_id/care-tasks/:task_id",
            get(get_care_task)
                .put(update_care_task)
                .delete(delete_care_task),
        )
        .route("/:plant_id/care-tasks/:task_id/log", post(log_care_task))
        .route(
            "/:plant_id/care-tasks/:task_id/archive",
            post(archive_care_task),
        )
        .route(
            "/:plant_id/care-tasks/:task_id/unarchive",
            post(unarchive_care_task),
        )
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListCareTasksQuery {
    include_archived: Option<bool>,
}

#[utoipa::path(
    get,
    path = "/plants/{plant_id}/care-tasks",
    params(
        ("plant_id" = Uuid, Path, description = "Plant ID"),
        ("includeArchived" = Option<bool>, Query, description = "Include archived tasks")
    ),
    responses(
        (status = 200, description = "List of care tasks for plant", body = CareTasksResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Plant not found"),
    ),
    tag = "care_tasks",
    security(("session" = []))
)]
async fn list_care_tasks(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path(plant_id): Path<Uuid>,
    Query(params): Query<ListCareTasksQuery>,
) -> Result<Json<CareTasksResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let response = db::list_care_tasks_for_plant(
        &app_state.pool,
        &plant_id,
        &user.id,
        params.include_archived.unwrap_or(false),
    )
    .await?;

    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/plants/{plant_id}/care-tasks",
    params(("plant_id" = Uuid, Path, description = "Plant ID")),
    request_body = CreateCareTaskRequest,
    responses(
        (status = 201, description = "Care task created", body = CareTaskWithStatus),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Plant not found"),
    ),
    tag = "care_tasks",
    security(("session" = []))
)]
async fn create_care_task(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path(plant_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<CreateCareTaskRequest>,
) -> Result<(StatusCode, Json<CareTaskWithStatus>)> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let task = db::create_care_task(&app_state.pool, &plant_id, &user.id, &payload).await?;
    Ok((StatusCode::CREATED, Json(task)))
}

#[utoipa::path(
    get,
    path = "/plants/{plant_id}/care-tasks/{task_id}",
    params(
        ("plant_id" = Uuid, Path, description = "Plant ID"),
        ("task_id" = Uuid, Path, description = "Care task ID"),
    ),
    responses(
        (status = 200, description = "Care task details", body = CareTaskWithStatus),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    ),
    tag = "care_tasks",
    security(("session" = []))
)]
async fn get_care_task(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path((_plant_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<CareTaskWithStatus>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let task = db::get_care_task(&app_state.pool, &task_id, &user.id).await?;
    Ok(Json(task))
}

#[utoipa::path(
    put,
    path = "/plants/{plant_id}/care-tasks/{task_id}",
    params(
        ("plant_id" = Uuid, Path, description = "Plant ID"),
        ("task_id" = Uuid, Path, description = "Care task ID"),
    ),
    request_body = UpdateCareTaskRequest,
    responses(
        (status = 200, description = "Care task updated", body = CareTaskWithStatus),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    ),
    tag = "care_tasks",
    security(("session" = []))
)]
async fn update_care_task(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path((_plant_id, task_id)): Path<(Uuid, Uuid)>,
    ValidatedJson(payload): ValidatedJson<UpdateCareTaskRequest>,
) -> Result<Json<CareTaskWithStatus>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let task = db::update_care_task(&app_state.pool, &task_id, &user.id, &payload).await?;
    Ok(Json(task))
}

#[utoipa::path(
    delete,
    path = "/plants/{plant_id}/care-tasks/{task_id}",
    params(
        ("plant_id" = Uuid, Path, description = "Plant ID"),
        ("task_id" = Uuid, Path, description = "Care task ID"),
    ),
    responses(
        (status = 204, description = "Care task deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    ),
    tag = "care_tasks",
    security(("session" = []))
)]
async fn delete_care_task(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path((_plant_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    db::delete_care_task(&app_state.pool, &task_id, &user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/plants/{plant_id}/care-tasks/{task_id}/log",
    params(
        ("plant_id" = Uuid, Path, description = "Plant ID"),
        ("task_id" = Uuid, Path, description = "Care task ID"),
    ),
    request_body = LogCareTaskRequest,
    responses(
        (status = 201, description = "Care task logged", body = LogCareTaskResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    ),
    tag = "care_tasks",
    security(("session" = []))
)]
async fn log_care_task(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path((_plant_id, task_id)): Path<(Uuid, Uuid)>,
    ValidatedJson(payload): ValidatedJson<LogCareTaskRequest>,
) -> Result<(StatusCode, Json<LogCareTaskResponse>)> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let (task, entry) = db::log_care_task(&app_state.pool, &task_id, &user.id, &payload).await?;
    Ok((
        StatusCode::CREATED,
        Json(LogCareTaskResponse { task, entry }),
    ))
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LogCareTaskResponse {
    pub task: CareTaskWithStatus,
    pub entry: TrackingEntry,
}

#[utoipa::path(
    post,
    path = "/plants/{plant_id}/care-tasks/{task_id}/archive",
    params(
        ("plant_id" = Uuid, Path, description = "Plant ID"),
        ("task_id" = Uuid, Path, description = "Care task ID"),
    ),
    responses(
        (status = 200, description = "Care task archived", body = CareTaskWithStatus),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    ),
    tag = "care_tasks",
    security(("session" = []))
)]
async fn archive_care_task(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path((_plant_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<CareTaskWithStatus>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let task = db::archive_care_task(&app_state.pool, &task_id, &user.id).await?;
    Ok(Json(task))
}

#[utoipa::path(
    post,
    path = "/plants/{plant_id}/care-tasks/{task_id}/unarchive",
    params(
        ("plant_id" = Uuid, Path, description = "Plant ID"),
        ("task_id" = Uuid, Path, description = "Care task ID"),
    ),
    responses(
        (status = 200, description = "Care task unarchived", body = CareTaskWithStatus),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    ),
    tag = "care_tasks",
    security(("session" = []))
)]
async fn unarchive_care_task(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path((_plant_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<CareTaskWithStatus>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let task = db::unarchive_care_task(&app_state.pool, &task_id, &user.id).await?;
    Ok(Json(task))
}

#[utoipa::path(
    put,
    path = "/plants/{plant_id}/care-tasks/reorder",
    params(("plant_id" = Uuid, Path, description = "Plant ID")),
    request_body = ReorderCareTasksRequest,
    responses(
        (status = 200, description = "Care tasks reordered", body = CareTasksResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Plant not found"),
    ),
    tag = "care_tasks",
    security(("session" = []))
)]
async fn reorder_care_tasks(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    Path(plant_id): Path<Uuid>,
    Json(payload): Json<ReorderCareTasksRequest>,
) -> Result<Json<CareTasksResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let response =
        db::reorder_care_tasks(&app_state.pool, &plant_id, &user.id, &payload.task_ids).await?;
    Ok(Json(response))
}
