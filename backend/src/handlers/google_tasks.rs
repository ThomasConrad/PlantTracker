use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::app_state::AppState;
use crate::database::{google_oauth, google_task_sync, plants as db_plants};
use crate::extractors::AuthenticatedUser;
use crate::models::google_oauth::{
    CreateGoogleTaskRequest, GoogleOAuthCallbackRequest, GoogleOAuthSuccessResponse,
    GoogleOAuthUrlResponse, GoogleTasksStatus, SyncPlantTasksRequest,
};
use crate::utils::errors::{AppError, Result};
use crate::utils::google_tasks::{
    create_plant_care_task, ensure_valid_token, exchange_code_for_tokens, generate_auth_url,
    generate_oauth_state, get_or_create_plant_care_task_list, get_task_status, GoogleTasksConfig,
};

/// Create Google Tasks routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth-url", get(get_google_auth_url))
        .route("/callback", get(handle_google_oauth_callback))
        .route("/store-tokens", post(store_google_tokens))
        .route("/status", get(get_google_tasks_status))
        .route("/disconnect", post(disconnect_google_tasks))
        .route("/sync-tasks", post(sync_plant_tasks))
        .route("/poll-completions", post(poll_completions))
        .route("/create-task", post(create_task))
}

/// Generate Google OAuth authorization URL
#[utoipa::path(
    get,
    path = "/google-tasks/auth-url",
    responses(
        (status = 200, description = "Google OAuth authorization URL", body = GoogleOAuthUrlResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Configuration error")
    ),
    tag = "google-tasks",
    security(
        ("session" = [])
    )
)]
pub async fn get_google_auth_url(
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<impl IntoResponse> {
    let config = GoogleTasksConfig::from_env()?;
    // Include user ID in the state parameter
    let state = format!("{}:{}", generate_oauth_state(), user.id);
    let auth_url = generate_auth_url(&config, &state);

    tracing::info!("Generated Google OAuth URL for user: {}", user.id);

    Ok(Json(GoogleOAuthUrlResponse { auth_url, state }))
}

/// Handle Google OAuth callback
#[utoipa::path(
    get,
    path = "/google-tasks/callback",
    params(
        ("code" = String, Query, description = "OAuth authorization code"),
        ("state" = Option<String>, Query, description = "OAuth state parameter")
    ),
    responses(
        (status = 302, description = "Redirect to frontend with success/error"),
        (status = 400, description = "Invalid callback parameters")
    ),
    tag = "google-tasks"
)]
pub async fn handle_google_oauth_callback(
    State(app_state): State<AppState>,
    Query(params): Query<GoogleOAuthCallbackRequest>,
) -> Result<impl IntoResponse> {
    tracing::info!("Handling Google OAuth callback with code: {}", params.code);

    let config = GoogleTasksConfig::from_env()?;
    tracing::info!("Google OAuth config loaded successfully");

    // Extract user ID from state parameter
    let user_id = if let Some(state) = &params.state {
        // URL decode the state parameter first
        let decoded_state = urlencoding::decode(state).map_err(|e| {
            tracing::error!("Failed to decode state parameter: {}", e);
            AppError::Authentication {
                message: "Invalid OAuth state parameter encoding".to_string(),
            }
        })?;

        // State format is "random_string:user_id"
        if let Some((_, user_id)) = decoded_state.split_once(':') {
            user_id.to_string()
        } else {
            tracing::error!("Invalid state parameter format: {}", decoded_state);
            return Err(AppError::Authentication {
                message: "Invalid OAuth state parameter".to_string(),
            });
        }
    } else {
        tracing::error!("Missing state parameter in OAuth callback");
        return Err(AppError::Authentication {
            message: "Missing OAuth state parameter".to_string(),
        });
    };

    tracing::info!("Extracted user ID from state: {}", user_id);

    // Exchange code for tokens
    let (access_token, refresh_token, expires_at) =
        exchange_code_for_tokens(&config, &params.code).await?;

    tracing::info!(
        "Successfully exchanged OAuth code for tokens for user: {}",
        user_id
    );

    // Store tokens directly in the database
    let scope = "https://www.googleapis.com/auth/tasks".to_string();

    google_oauth::save_oauth_token(
        &app_state.pool,
        &user_id,
        &access_token,
        refresh_token.as_deref(),
        expires_at,
        &scope,
    )
    .await?;

    tracing::info!("Stored Google OAuth tokens for user: {}", user_id);

    // Notify the token refresh scheduler about the new token
    app_state.notify_token_added();

    // Redirect back to calendar settings (relative path works since backend serves frontend)
    let redirect_url = "/calendar-settings";

    tracing::info!(
        "Google OAuth callback successful, redirecting to: {}",
        redirect_url
    );
    Ok(Redirect::temporary(redirect_url))
}

/// Store Google OAuth tokens (called by frontend after callback)
#[utoipa::path(
    post,
    path = "/google-tasks/store-tokens",
    request_body = StoreTokensRequest,
    responses(
        (status = 200, description = "Tokens stored successfully", body = GoogleOAuthSuccessResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Failed to store tokens")
    ),
    tag = "google-tasks",
    security(
        ("session" = [])
    )
)]
pub async fn store_google_tokens(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(request): Json<StoreTokensRequest>,
) -> Result<impl IntoResponse> {
    let expires_at = if request.expires_at > 0 {
        Some(chrono::DateTime::from_timestamp(request.expires_at, 0).unwrap_or_else(Utc::now))
    } else {
        None
    };

    let scope = "https://www.googleapis.com/auth/tasks".to_string();

    google_oauth::save_oauth_token(
        &app_state.pool,
        &user.id,
        &request.access_token,
        request.refresh_token.as_deref(),
        expires_at,
        &scope,
    )
    .await?;

    tracing::info!("Stored Google OAuth tokens for user: {}", user.id);

    // Notify the token refresh scheduler about the new token
    app_state.notify_token_added();

    Ok(Json(GoogleOAuthSuccessResponse {
        success: true,
        message: "Google Tasks integration configured successfully".to_string(),
        connected_at: Utc::now(),
        scopes: vec![scope],
    }))
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct StoreTokensRequest {
    /// The access token from Google OAuth
    pub access_token: String,
    /// The refresh token from Google OAuth (optional)
    pub refresh_token: Option<String>,
    /// Unix timestamp when the token expires
    pub expires_at: i64,
}

/// Get Google Tasks connection status
#[utoipa::path(
    get,
    path = "/google-tasks/status",
    responses(
        (status = 200, description = "Google Tasks connection status", body = GoogleTasksStatus),
        (status = 401, description = "Unauthorized")
    ),
    tag = "google-tasks",
    security(
        ("session" = [])
    )
)]
pub async fn get_google_tasks_status(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<impl IntoResponse> {
    let token = google_oauth::get_oauth_token(&app_state.pool, &user.id).await?;

    let status = match token {
        Some(token) => {
            // Check if token is actually valid (not expired)
            let is_valid = if let Some(expires_at) = token.expires_at {
                // Consider token expired if it expires within the next 5 minutes
                expires_at > chrono::Utc::now() + chrono::Duration::minutes(5)
            } else {
                // If no expiration time, assume it's valid
                true
            };

            GoogleTasksStatus {
                connected: is_valid,
                connected_at: Some(token.created_at),
                scopes: Some(
                    token
                        .scope
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect(),
                ),
                expires_at: token.expires_at,
            }
        }
        None => GoogleTasksStatus {
            connected: false,
            connected_at: None,
            scopes: None,
            expires_at: None,
        },
    };

    Ok(Json(status))
}

/// Disconnect Google Tasks integration
#[utoipa::path(
    post,
    path = "/google-tasks/disconnect",
    responses(
        (status = 200, description = "Google Tasks disconnected successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "No Google Tasks connection found")
    ),
    tag = "google-tasks",
    security(
        ("session" = [])
    )
)]
pub async fn disconnect_google_tasks(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<impl IntoResponse> {
    google_oauth::delete_oauth_token(&app_state.pool, &user.id).await?;

    tracing::info!("Disconnected Google Tasks for user: {}", user.id);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Google Tasks disconnected successfully"
    })))
}

/// Sync plant care tasks to Google Tasks
#[utoipa::path(
    post,
    path = "/google-tasks/sync-tasks",
    request_body = SyncPlantTasksRequest,
    responses(
        (status = 200, description = "Plant tasks synced successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "No Google Tasks connection found"),
        (status = 500, description = "Failed to sync tasks")
    ),
    tag = "google-tasks",
    security(
        ("session" = [])
    )
)]
pub async fn sync_plant_tasks(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(request): Json<SyncPlantTasksRequest>,
) -> Result<impl IntoResponse> {
    let config = GoogleTasksConfig::from_env()?;
    let token = ensure_valid_token(&app_state.pool, &user.id, &config).await?;

    // Get or create the "Plant Care" task list
    let task_list_id = get_or_create_plant_care_task_list(&token).await?;

    // Get user's plants
    let (plants, _) =
        db_plants::list_plants_for_user(&app_state.pool, &user.id, 1000, 0, None).await?;

    let days_ahead = request.days_ahead.unwrap_or(365);
    let base_url =
        std::env::var("BASE_URL").unwrap_or_else(|_| "https://your-domain.com".to_string());

    let mut created_tasks = 0;
    let mut skipped_tasks = 0;
    let now = Utc::now();
    let end_date = now + chrono::Duration::days(days_ahead as i64);

    for plant in &plants {
        for task_with_status in &plant.care_tasks {
            let ct = &task_with_status.task;
            let Some(interval) = ct.interval_days else {
                continue;
            };
            if interval <= 0 {
                continue;
            }

            let last = ct
                .last_performed
                .unwrap_or_else(|| now - chrono::Duration::days(interval as i64));

            let mut next = last + chrono::Duration::days(interval as i64);
            while next <= end_date && next >= now {
                let due_date = next.format("%Y-%m-%d").to_string();

                // Check if we already synced this task+date
                let existing = google_task_sync::get_synced_task(
                    &app_state.pool,
                    &user.id,
                    &ct.id.to_string(),
                    &due_date,
                )
                .await?;

                if existing.is_some() {
                    skipped_tasks += 1;
                    next += chrono::Duration::days(interval as i64);
                    continue;
                }

                match create_plant_care_task(&token, plant, ct, next, &base_url, &task_list_id)
                    .await
                {
                    Ok(google_task_id) => {
                        // Track the synced task
                        let _ = google_task_sync::insert_synced_task(
                            &app_state.pool,
                            &user.id,
                            &plant.id.to_string(),
                            &ct.id.to_string(),
                            &google_task_id,
                            &task_list_id,
                            &due_date,
                        )
                        .await;
                        created_tasks += 1;
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to create {} task for {}: {}",
                            ct.name,
                            plant.name,
                            e
                        )
                    }
                }
                next += chrono::Duration::days(interval as i64);
            }
        }
    }

    tracing::info!(
        "Synced plant tasks for user {}: {} created, {} skipped (already exist)",
        user.id,
        created_tasks,
        skipped_tasks
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Created {} new plant care tasks ({} already synced)", created_tasks, skipped_tasks),
        "tasks_created": created_tasks,
        "tasks_skipped": skipped_tasks,
        "plants_processed": plants.len(),
        "days_ahead": days_ahead
    })))
}

/// Poll Google Tasks for completions and mark care tasks as done in Planty
#[utoipa::path(
    post,
    path = "/google-tasks/poll-completions",
    responses(
        (status = 200, description = "Polled completions successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "No Google Tasks connection found"),
        (status = 500, description = "Failed to poll completions")
    ),
    tag = "google-tasks",
    security(
        ("session" = [])
    )
)]
pub async fn poll_completions(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<impl IntoResponse> {
    // Check for incomplete tasks first (avoids needing Google config if nothing to do)
    let incomplete = google_task_sync::get_incomplete_tasks(&app_state.pool, &user.id).await?;

    if incomplete.is_empty() {
        return Ok(Json(serde_json::json!({
            "success": true,
            "message": "No tasks to check",
            "completed": 0,
            "checked": 0
        })));
    }

    let config = GoogleTasksConfig::from_env()?;
    let token = ensure_valid_token(&app_state.pool, &user.id, &config).await?;

    let mut completed_count = 0;

    for record in &incomplete {
        // Check status in Google Tasks
        match get_task_status(&token, &record.google_task_list_id, &record.google_task_id).await {
            Ok(status) if status == "completed" => {
                // Mark as completed in our sync table
                google_task_sync::mark_task_completed(&app_state.pool, &record.google_task_id)
                    .await?;

                // Log the care task as performed in Planty
                if let Ok(task_uuid) = uuid::Uuid::parse_str(&record.care_task_id) {
                    let log_req = crate::models::care_task::LogCareTaskRequest {
                        timestamp: None,
                        value: None,
                        notes: Some("Completed via Google Tasks".to_string()),
                        photo_ids: None,
                    };
                    let _ = crate::database::care_tasks::log_care_task(
                        &app_state.pool,
                        &task_uuid,
                        &user.id,
                        &log_req,
                    )
                    .await;
                }

                completed_count += 1;
            }
            Ok(_) => {
                // Still needsAction, skip
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to check task {} status: {}",
                    record.google_task_id,
                    e
                );
            }
        }
    }

    tracing::info!(
        "Poll completions for user {}: {}/{} tasks completed",
        user.id,
        completed_count,
        incomplete.len()
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("{} task(s) completed in Google Tasks and synced back", completed_count),
        "completed": completed_count,
        "checked": incomplete.len()
    })))
}

/// Create a single task
#[utoipa::path(
    post,
    path = "/google-tasks/create-task",
    request_body = CreateGoogleTaskRequest,
    responses(
        (status = 200, description = "Task created successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "No Google Tasks connection found"),
        (status = 500, description = "Failed to create task")
    ),
    tag = "google-tasks",
    security(
        ("session" = [])
    )
)]
pub async fn create_task(
    State(app_state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(request): Json<CreateGoogleTaskRequest>,
) -> Result<impl IntoResponse> {
    let config = GoogleTasksConfig::from_env()?;
    let token = ensure_valid_token(&app_state.pool, &user.id, &config).await?;

    // Get or create task list
    let task_list_id = if let Some(list_id) = request.task_list_id {
        list_id
    } else {
        get_or_create_plant_care_task_list(&token).await?
    };

    let client = reqwest::Client::new();

    let task_data = serde_json::json!({
        "title": request.title,
        "notes": request.notes,
        "due": request.due_time.to_rfc3339(),
        "status": "needsAction"
    });

    let response = client
        .post(format!(
            "{}/tasks/v1/lists/{}/tasks",
            crate::utils::google_tasks::tasks_api_base_url(),
            task_list_id
        ))
        .header("Authorization", format!("Bearer {}", token.access_token))
        .header("Content-Type", "application/json")
        .json(&task_data)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to create task: {}", e);
            AppError::External {
                message: "Failed to create Google Task".to_string(),
            }
        })?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        tracing::error!("Google Tasks API error: {}", error_text);
        return Err(AppError::External {
            message: "Google Tasks API request failed".to_string(),
        });
    }

    let result: serde_json::Value = response.json().await.map_err(|e| {
        tracing::error!("Failed to parse Google Tasks response: {}", e);
        AppError::External {
            message: "Invalid response from Google Tasks".to_string(),
        }
    })?;

    let task_id = result["id"]
        .as_str()
        .ok_or_else(|| AppError::External {
            message: "No task ID returned from Google Tasks".to_string(),
        })?
        .to_string();

    tracing::info!("Created task for user {}: {}", user.id, task_id);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Task created successfully",
        "task_id": task_id,
        "task_list_id": task_list_id
    })))
}
