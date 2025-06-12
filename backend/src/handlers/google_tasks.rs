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
use crate::auth::AuthSession;
use crate::database::{google_oauth, plants as db_plants};
use crate::models::google_oauth::{
    CreateGoogleTaskRequest, GoogleOAuthCallbackRequest, GoogleOAuthSuccessResponse,
    GoogleOAuthUrlResponse, GoogleTasksStatus, SyncPlantTasksRequest,
};
use crate::utils::errors::{AppError, Result};
use crate::utils::google_tasks::{
    create_plant_care_task, ensure_valid_token, exchange_code_for_tokens, generate_auth_url,
    generate_oauth_state, get_or_create_plant_care_task_list, GoogleTasksConfig,
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
pub async fn get_google_auth_url(auth_session: AuthSession) -> Result<impl IntoResponse> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

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

    // Redirect back to calendar settings without any parameters
    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let redirect_url = format!("{}/calendar-settings", frontend_url);

    tracing::info!("Google OAuth callback successful, redirecting to: {}", redirect_url);
    Ok(Redirect::temporary(&redirect_url))
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
    auth_session: AuthSession,
    Json(request): Json<StoreTokensRequest>,
) -> Result<impl IntoResponse> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

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
    auth_session: AuthSession,
) -> Result<impl IntoResponse> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

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
    auth_session: AuthSession,
) -> Result<impl IntoResponse> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

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
    auth_session: AuthSession,
    Json(request): Json<SyncPlantTasksRequest>,
) -> Result<impl IntoResponse> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let config = GoogleTasksConfig::from_env()?;
    let token = ensure_valid_token(&app_state.pool, &user.id, &config).await?;

    // Get or create the "Plant Care" task list
    let task_list_id = get_or_create_plant_care_task_list(&token).await?;

    // Get user's plants
    let (plants, _) = db_plants::list_plants_for_user(&app_state.pool, &user.id, 1000, 0, None).await?;

    let days_ahead = request.days_ahead.unwrap_or(365);
    let base_url =
        std::env::var("BASE_URL").unwrap_or_else(|_| "https://your-domain.com".to_string());

    let mut created_tasks = 0;
    let now = Utc::now();
    let end_date = now + chrono::Duration::days(days_ahead as i64);

    for plant in &plants {
        // Generate watering tasks
        let last_watered = plant
            .last_watered
            .unwrap_or_else(|| now - chrono::Duration::days(plant.watering_interval_days as i64));

        let mut next_watering =
            last_watered + chrono::Duration::days(plant.watering_interval_days as i64);
        while next_watering <= end_date && next_watering >= now {
            match create_plant_care_task(
                &token,
                plant,
                "watering",
                next_watering,
                &base_url,
                &task_list_id,
            )
            .await
            {
                Ok(_task_id) => created_tasks += 1,
                Err(e) => {
                    tracing::error!("Failed to create watering task for {}: {}", plant.name, e)
                }
            }
            next_watering += chrono::Duration::days(plant.watering_interval_days as i64);
        }

        // Generate fertilizing tasks
        let last_fertilized = plant.last_fertilized.unwrap_or_else(|| {
            now - chrono::Duration::days(plant.fertilizing_interval_days as i64)
        });

        let mut next_fertilizing =
            last_fertilized + chrono::Duration::days(plant.fertilizing_interval_days as i64);
        while next_fertilizing <= end_date && next_fertilizing >= now {
            match create_plant_care_task(
                &token,
                plant,
                "fertilizing",
                next_fertilizing,
                &base_url,
                &task_list_id,
            )
            .await
            {
                Ok(_task_id) => created_tasks += 1,
                Err(e) => tracing::error!(
                    "Failed to create fertilizing task for {}: {}",
                    plant.name,
                    e
                ),
            }
            next_fertilizing += chrono::Duration::days(plant.fertilizing_interval_days as i64);
        }
    }

    tracing::info!(
        "Synced {} plant care tasks to Google Tasks for user: {}",
        created_tasks,
        user.id
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Created {} plant care tasks in your Google Tasks", created_tasks),
        "tasks_created": created_tasks,
        "plants_processed": plants.len(),
        "days_ahead": days_ahead
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
    auth_session: AuthSession,
    Json(request): Json<CreateGoogleTaskRequest>,
) -> Result<impl IntoResponse> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

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
            "https://tasks.googleapis.com/tasks/v1/lists/{}/tasks",
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
