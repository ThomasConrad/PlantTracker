use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::Deserialize;

use crate::auth::AuthSession;
use crate::database::{google_oauth, plants as db_plants, DatabasePool};
use crate::models::google_oauth::{
    GoogleOAuthCallbackRequest, GoogleOAuthSuccessResponse, GoogleOAuthUrlResponse,
    GoogleCalendarStatus, SyncPlantRemindersRequest, CreateGoogleCalendarEventRequest,
};
use crate::utils::errors::{AppError, Result};
use crate::utils::google_calendar::{
    GoogleCalendarConfig, generate_auth_url, exchange_code_for_tokens, 
    generate_oauth_state, ensure_valid_token, create_calendar_hub,
};

/// Create Google Calendar routes
pub fn routes() -> Router<DatabasePool> {
    Router::new()
        .route("/auth-url", get(get_google_auth_url))
        .route("/callback", get(handle_google_oauth_callback))
        .route("/store-tokens", post(store_google_tokens))
        .route("/status", get(get_google_calendar_status))
        .route("/disconnect", post(disconnect_google_calendar))
        .route("/sync-reminders", post(sync_plant_reminders))
        .route("/create-event", post(create_calendar_event))
}

/// Generate Google OAuth authorization URL
#[utoipa::path(
    get,
    path = "/google-calendar/auth-url",
    responses(
        (status = 200, description = "Google OAuth authorization URL", body = GoogleOAuthUrlResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Configuration error")
    ),
    tag = "google-calendar",
    security(
        ("session" = [])
    )
)]
pub async fn get_google_auth_url(
    auth_session: AuthSession,
) -> Result<impl IntoResponse> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let config = GoogleCalendarConfig::from_env()?;
    let state = generate_oauth_state();
    let auth_url = generate_auth_url(&config, &state);

    tracing::info!("Generated Google OAuth URL for user: {}", user.id);

    Ok(Json(GoogleOAuthUrlResponse {
        auth_url,
        state,
    }))
}

/// Handle Google OAuth callback
#[utoipa::path(
    get,
    path = "/google-calendar/callback",
    params(
        ("code" = String, Query, description = "OAuth authorization code"),
        ("state" = Option<String>, Query, description = "OAuth state parameter")
    ),
    responses(
        (status = 302, description = "Redirect to frontend with success/error"),
        (status = 400, description = "Invalid callback parameters")
    ),
    tag = "google-calendar"
)]
pub async fn handle_google_oauth_callback(
    State(_pool): State<DatabasePool>,
    Query(params): Query<GoogleOAuthCallbackRequest>,
) -> Result<impl IntoResponse> {
    let config = GoogleCalendarConfig::from_env()?;
    
    // Exchange code for tokens
    let (access_token, refresh_token, expires_at) = 
        exchange_code_for_tokens(&config, &params.code).await?;

    // For now, we need to get the user ID from the state or session
    // In a real implementation, you'd want to store the state with the user ID
    // For this demo, we'll redirect to the frontend with the tokens as query params
    // The frontend should then call an authenticated endpoint to store the tokens
    
    let frontend_url = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:5173".to_string());
    
    let redirect_url = format!(
        "{}/calendar-settings?google_auth=success&access_token={}&refresh_token={}&expires_at={}",
        frontend_url,
        urlencoding::encode(&access_token),
        urlencoding::encode(&refresh_token.unwrap_or_default()),
        expires_at.map(|dt| dt.timestamp()).unwrap_or(0)
    );

    tracing::info!("Google OAuth callback successful, redirecting to frontend");
    Ok(Redirect::temporary(&redirect_url))
}

/// Store Google OAuth tokens (called by frontend after callback)
#[utoipa::path(
    post,
    path = "/google-calendar/store-tokens",
    request_body = StoreTokensRequest,
    responses(
        (status = 200, description = "Tokens stored successfully", body = GoogleOAuthSuccessResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Failed to store tokens")
    ),
    tag = "google-calendar",
    security(
        ("session" = [])
    )
)]
pub async fn store_google_tokens(
    State(pool): State<DatabasePool>,
    auth_session: AuthSession,
    Json(request): Json<StoreTokensRequest>,
) -> Result<impl IntoResponse> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let expires_at = if request.expires_at > 0 {
        Some(chrono::DateTime::from_timestamp(request.expires_at, 0)
            .unwrap_or_else(|| Utc::now()))
    } else {
        None
    };

    let scope = "https://www.googleapis.com/auth/calendar.events".to_string();
    
    google_oauth::save_oauth_token(
        &pool,
        &user.id,
        &request.access_token,
        request.refresh_token.as_deref(),
        expires_at,
        &scope,
    ).await?;

    tracing::info!("Stored Google OAuth tokens for user: {}", user.id);

    Ok(Json(GoogleOAuthSuccessResponse {
        success: true,
        message: "Google Calendar integration configured successfully".to_string(),
        connected_at: Utc::now(),
        scopes: vec![scope],
    }))
}

#[derive(Deserialize)]
pub struct StoreTokensRequest {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64,
}

/// Get Google Calendar connection status
#[utoipa::path(
    get,
    path = "/google-calendar/status",
    responses(
        (status = 200, description = "Google Calendar connection status", body = GoogleCalendarStatus),
        (status = 401, description = "Unauthorized")
    ),
    tag = "google-calendar",
    security(
        ("session" = [])
    )
)]
pub async fn get_google_calendar_status(
    State(pool): State<DatabasePool>,
    auth_session: AuthSession,
) -> Result<impl IntoResponse> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let token = google_oauth::get_oauth_token(&pool, &user.id).await?;

    let status = match token {
        Some(token) => GoogleCalendarStatus {
            connected: true,
            connected_at: Some(token.created_at),
            scopes: Some(token.scope.split(',').map(|s| s.trim().to_string()).collect()),
            expires_at: token.expires_at,
        },
        None => GoogleCalendarStatus {
            connected: false,
            connected_at: None,
            scopes: None,
            expires_at: None,
        },
    };

    Ok(Json(status))
}

/// Disconnect Google Calendar integration
#[utoipa::path(
    post,
    path = "/google-calendar/disconnect",
    responses(
        (status = 200, description = "Google Calendar disconnected successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "No Google Calendar connection found")
    ),
    tag = "google-calendar",
    security(
        ("session" = [])
    )
)]
pub async fn disconnect_google_calendar(
    State(pool): State<DatabasePool>,
    auth_session: AuthSession,
) -> Result<impl IntoResponse> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    google_oauth::delete_oauth_token(&pool, &user.id).await?;

    tracing::info!("Disconnected Google Calendar for user: {}", user.id);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Google Calendar disconnected successfully"
    })))
}

/// Sync plant care reminders to Google Calendar
#[utoipa::path(
    post,
    path = "/google-calendar/sync-reminders",
    request_body = SyncPlantRemindersRequest,
    responses(
        (status = 200, description = "Plant reminders synced successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "No Google Calendar connection found"),
        (status = 500, description = "Failed to sync reminders")
    ),
    tag = "google-calendar",
    security(
        ("session" = [])
    )
)]
pub async fn sync_plant_reminders(
    State(pool): State<DatabasePool>,
    auth_session: AuthSession,
    Json(request): Json<SyncPlantRemindersRequest>,
) -> Result<impl IntoResponse> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let config = GoogleCalendarConfig::from_env()?;
    let token = ensure_valid_token(&pool, &user.id, &config).await?;
    let hub = create_calendar_hub(&token).await?;

    // Get user's plants
    let (plants, _) = db_plants::list_plants_for_user(&pool, &user.id, 1000, 0, None).await?;
    
    let days_ahead = request.days_ahead.unwrap_or(365);
    let base_url = std::env::var("BASE_URL")
        .unwrap_or_else(|_| "https://your-domain.com".to_string());

    let mut created_events = 0;
    let now = Utc::now();
    let end_date = now + chrono::Duration::days(days_ahead as i64);

    for plant in &plants {
        // Generate watering events
        let last_watered = plant.last_watered
            .unwrap_or_else(|| now - chrono::Duration::days(plant.watering_interval_days as i64));
        
        let mut next_watering = last_watered + chrono::Duration::days(plant.watering_interval_days as i64);
        while next_watering <= end_date && next_watering >= now {
            match crate::utils::google_calendar::create_plant_care_event(
                &hub, plant, "watering", next_watering, &base_url
            ).await {
                Ok(_event_id) => created_events += 1,
                Err(e) => tracing::error!("Failed to create watering event for {}: {}", plant.name, e),
            }
            next_watering = next_watering + chrono::Duration::days(plant.watering_interval_days as i64);
        }

        // Generate fertilizing events
        let last_fertilized = plant.last_fertilized
            .unwrap_or_else(|| now - chrono::Duration::days(plant.fertilizing_interval_days as i64));
        
        let mut next_fertilizing = last_fertilized + chrono::Duration::days(plant.fertilizing_interval_days as i64);
        while next_fertilizing <= end_date && next_fertilizing >= now {
            match crate::utils::google_calendar::create_plant_care_event(
                &hub, plant, "fertilizing", next_fertilizing, &base_url
            ).await {
                Ok(_event_id) => created_events += 1,
                Err(e) => tracing::error!("Failed to create fertilizing event for {}: {}", plant.name, e),
            }
            next_fertilizing = next_fertilizing + chrono::Duration::days(plant.fertilizing_interval_days as i64);
        }
    }

    tracing::info!("Synced {} plant care events to Google Calendar for user: {}", created_events, user.id);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Created {} plant care reminders in your Google Calendar", created_events),
        "events_created": created_events,
        "plants_processed": plants.len(),
        "days_ahead": days_ahead
    })))
}

/// Create a single calendar event
#[utoipa::path(
    post,
    path = "/google-calendar/create-event",
    request_body = CreateGoogleCalendarEventRequest,
    responses(
        (status = 200, description = "Calendar event created successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "No Google Calendar connection found"),
        (status = 500, description = "Failed to create event")
    ),
    tag = "google-calendar",
    security(
        ("session" = [])
    )
)]
pub async fn create_calendar_event(
    State(pool): State<DatabasePool>,
    auth_session: AuthSession,
    Json(request): Json<CreateGoogleCalendarEventRequest>,
) -> Result<impl IntoResponse> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let config = GoogleCalendarConfig::from_env()?;
    let token = ensure_valid_token(&pool, &user.id, &config).await?;
    let hub = create_calendar_hub(&token).await?;

    use google_calendar3::api::{Event, EventDateTime};

    let event = Event {
        summary: Some(request.summary),
        description: request.description,
        start: Some(EventDateTime {
            date_time: Some(request.start_time.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
            time_zone: Some("UTC".to_string()),
            ..Default::default()
        }),
        end: Some(EventDateTime {
            date_time: Some(request.end_time.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
            time_zone: Some("UTC".to_string()),
            ..Default::default()
        }),
        location: request.location,
        ..Default::default()
    };

    let calendar_id = request.calendar_id.as_deref().unwrap_or("primary");
    
    let result = hub
        .events()
        .insert(event, calendar_id)
        .doit()
        .await
        .map_err(|e| {
            tracing::error!("Failed to create calendar event: {}", e);
            AppError::External {
                message: "Failed to create Google Calendar event".to_string(),
            }
        })?;

    let event_id = result.1.id.ok_or_else(|| AppError::External {
        message: "No event ID returned from Google Calendar".to_string(),
    })?;

    tracing::info!("Created calendar event for user {}: {}", user.id, event_id);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Calendar event created successfully",
        "event_id": event_id,
        "calendar_id": calendar_id
    })))
}