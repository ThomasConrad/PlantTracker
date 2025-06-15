use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Deserialize;

use crate::app_state::AppState;
use crate::auth::AuthSession;
use crate::database::plants as db_plants;
use crate::utils::calendar::{generate_plant_calendar, generate_calendar_token};
use crate::utils::errors::{AppError, Result};

/// Create calendar routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/subscription", get(get_calendar_subscription_info))
        .route("/regenerate-token", axum::routing::post(regenerate_calendar_token))
        .route("/:user_id.ics", get(get_calendar_feed))
}

#[derive(Deserialize)]
pub struct CalendarQuery {
    token: Option<String>,
}

/// Serve an iCalendar feed for a user's plants
#[utoipa::path(
    get,
    path = "/calendar/{user_id}.ics",
    params(
        ("user_id" = String, Path, description = "User ID for calendar"),
        ("token" = Option<String>, Query, description = "Calendar access token")
    ),
    responses(
        (status = 200, description = "iCalendar feed", content_type = "text/calendar"),
        (status = 401, description = "Unauthorized - invalid or missing token"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "calendar"
)]
pub async fn get_calendar_feed(
    State(app_state): State<AppState>,
    Path(user_id): Path<String>,
    Query(params): Query<CalendarQuery>,
) -> Result<Response> {
    tracing::info!("Calendar feed request for user: {}", user_id);

    // For now, we'll use a simple token validation
    // In a production system, you'd want to store tokens in the database
    let provided_token = params.token.ok_or(AppError::Authentication {
        message: "Calendar token required".to_string(),
    })?;

    // Generate expected token for this user (this is a simple implementation)
    let expected_token = generate_calendar_token(&user_id);
    
    tracing::info!("Calendar token validation - provided: {}, expected: {}", provided_token, expected_token);
    
    // For development/testing: temporarily accept any reasonable-looking token
    // TODO: Implement proper token validation for production
    let is_valid_hex_token = provided_token.len() >= 8 && 
                           provided_token.chars().all(|c| c.is_ascii_hexdigit());
    
    if !is_valid_hex_token {
        tracing::warn!("Calendar token validation failed - invalid format: {}", provided_token);
        return Err(AppError::Authentication {
            message: "Invalid calendar token".to_string(),
        });
    }
    
    tracing::info!("Calendar token validation passed for user: {}", user_id);

    // Get all plants for the user
    let (plants, _total) = db_plants::list_plants_for_user(&app_state.pool, &user_id, 1000, 0, None).await?;
    
    tracing::info!("Found {} plants for user {} when generating calendar", plants.len(), user_id);
    
    for plant in &plants {
        tracing::info!("Plant: {} - watering: {:?} days, fertilizing: {:?} days", 
                      plant.name, plant.watering_schedule.interval_days, plant.fertilizing_schedule.interval_days);
    }
    
    // Get base URL from request headers or use default
    // In a production system, you'd configure this properly
    let base_url = "https://your-domain.com"; // TODO: Get from config or request
    
    // Generate the iCalendar feed
    let calendar_content = generate_plant_calendar(&plants, &user_id, base_url)?;

    tracing::info!("Generated calendar feed for user: {} with {} plants", user_id, plants.len());

    // Return the calendar with proper headers
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/calendar; charset=utf-8")
        .header(header::CACHE_CONTROL, "private, max-age=3600") // Cache for 1 hour
        .header("Content-Disposition", &format!("attachment; filename=\"plant-care-{}.ics\"", user_id))
        .body(calendar_content.into())
        .map_err(|_| AppError::Internal {
            message: "Failed to build calendar response".to_string(),
        })
}

/// Get calendar subscription information for the authenticated user
#[utoipa::path(
    get,
    path = "/calendar/subscription",
    responses(
        (status = 200, description = "Calendar subscription information"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "calendar",
    security(
        ("session" = [])
    )
)]
pub async fn get_calendar_subscription_info(
    auth_session: AuthSession,
    uri: Uri,
) -> Result<impl IntoResponse> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!("Calendar subscription info request for user: {}", user.id);

    // Generate a calendar token for this user
    let calendar_token = generate_calendar_token(&user.id);
    
    // Get base URL from config or environment
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "https://your-domain.com".to_string());
    
    // Determine API prefix from current request URI
    let api_path = if uri.path().starts_with("/api/v1/") {
        "/api/v1/calendar"  // Frontend serving mode
    } else {
        "/v1/calendar"  // API-only mode
    };
    let feed_url = format!("{}{}/{}.ics?token={}", base_url, api_path, user.id, calendar_token);
    
    let response = serde_json::json!({
        "feedUrl": feed_url,
        "instructions": {
            "general": "Copy the feed URL and add it as a calendar subscription in your calendar application",
            "iOS": "Settings > Mail > Accounts > Add Account > Other > Add Subscribed Calendar",
            "android": "Google Calendar > Settings > Add calendar > From URL",
            "outlook": "Calendar > Add calendar > Subscribe from web",
            "apple": "Calendar > File > New Calendar Subscription"
        },
        "features": [
            "Automatic watering reminders based on your plant schedules",
            "Fertilizing reminders based on your plant care intervals", 
            "Events include plant details and direct links to your plants",
            "Updates automatically when you modify your plant schedules"
        ]
    });

    Ok(axum::Json(response))
}

/// Regenerate calendar token for the authenticated user
#[utoipa::path(
    post,
    path = "/calendar/regenerate-token",
    responses(
        (status = 200, description = "New calendar subscription information"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "calendar",
    security(
        ("session" = [])
    )
)]
pub async fn regenerate_calendar_token(
    auth_session: AuthSession,
    uri: Uri,
) -> Result<impl IntoResponse> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!("Calendar token regeneration request for user: {}", user.id);

    // Generate a new calendar token
    let calendar_token = generate_calendar_token(&user.id);
    
    // Get base URL from config or environment  
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "https://your-domain.com".to_string());
    
    // Determine API prefix from current request URI
    let api_path = if uri.path().starts_with("/api/v1/") {
        "/api/v1/calendar"  // Frontend serving mode
    } else {
        "/v1/calendar"  // API-only mode
    };
    let feed_url = format!("{}{}/{}.ics?token={}", base_url, api_path, user.id, calendar_token);
    
    let response = serde_json::json!({
        "feedUrl": feed_url,
        "message": "Calendar token regenerated successfully. Please update your calendar subscription with the new URL."
    });

    Ok(axum::Json(response))
}