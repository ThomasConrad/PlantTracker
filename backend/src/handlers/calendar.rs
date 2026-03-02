use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use sqlx::Row;

use crate::app_state::AppState;
use crate::auth::AuthSession;
use crate::database::plants as db_plants;
use crate::utils::calendar::{generate_calendar_token, generate_plant_calendar};
use crate::utils::errors::{AppError, Result};

/// Extract base URL from request headers
fn get_base_url_from_headers(headers: &HeaderMap, _uri: &Uri) -> String {
    // Try to get the host from headers
    let host = headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost:3000");

    // Check for forwarded protocol headers (common in reverse proxies)
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|h| h.to_str().ok())
        .or_else(|| {
            headers
                .get("x-forwarded-protocol")
                .and_then(|h| h.to_str().ok())
        })
        .unwrap_or_else(|| {
            // Default to https for production domains, http for localhost
            if host.starts_with("localhost") || host.starts_with("127.0.0.1") {
                "http"
            } else {
                "https"
            }
        });

    format!("{}://{}", scheme, host)
}

/// Create calendar routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/subscription", get(get_calendar_subscription_info))
        .route(
            "/regenerate-token",
            axum::routing::post(regenerate_calendar_token),
        )
        .route("/:user_id.ics", get(get_calendar_feed))
}

#[derive(Deserialize)]
pub struct CalendarQuery {
    token: Option<String>,
}

async fn get_or_create_calendar_token(pool: &sqlx::SqlitePool, user_id: &str) -> Result<String> {
    let existing_token = sqlx::query("SELECT calendar_token FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .and_then(|row| {
            row.try_get::<Option<String>, _>("calendar_token")
                .ok()
                .flatten()
        });

    if let Some(token) = existing_token {
        return Ok(token);
    }

    let token = generate_calendar_token();
    let now = chrono::Utc::now().to_rfc3339();

    let updated = sqlx::query(
        "UPDATE users SET calendar_token = ?, calendar_token_updated_at = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&token)
    .bind(&now)
    .bind(&now)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    if updated.rows_affected() != 1 {
        return Err(AppError::NotFound {
            resource: format!("User with id {}", user_id),
        });
    }

    Ok(token)
}

async fn rotate_calendar_token(pool: &sqlx::SqlitePool, user_id: &str) -> Result<String> {
    let token = generate_calendar_token();
    let now = chrono::Utc::now().to_rfc3339();

    let updated = sqlx::query(
        "UPDATE users SET calendar_token = ?, calendar_token_updated_at = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&token)
    .bind(&now)
    .bind(&now)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    if updated.rows_affected() != 1 {
        return Err(AppError::NotFound {
            resource: format!("User with id {}", user_id),
        });
    }

    Ok(token)
}

async fn verify_calendar_token(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    provided_token: &str,
) -> Result<bool> {
    let stored_token = sqlx::query("SELECT calendar_token FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .and_then(|row| {
            row.try_get::<Option<String>, _>("calendar_token")
                .ok()
                .flatten()
        });

    Ok(stored_token.is_some_and(|token| token == provided_token))
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
    Path(user_id_with_ext): Path<String>,
    Query(params): Query<CalendarQuery>,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Response> {
    // Extract user_id by removing .ics extension if present
    let user_id = user_id_with_ext
        .strip_suffix(".ics")
        .unwrap_or(&user_id_with_ext);
    tracing::info!("Calendar feed request for user: {}", user_id);

    let provided_token = params.token.ok_or(AppError::Authentication {
        message: "Calendar token required".to_string(),
    })?;

    if !verify_calendar_token(&app_state.pool, user_id, &provided_token).await? {
        tracing::warn!("Calendar token validation failed for user: {}", user_id);
        return Err(AppError::Authentication {
            message: "Invalid calendar token".to_string(),
        });
    }

    tracing::info!("Calendar token validation passed for user: {}", user_id);

    // Get all plants for the user
    let (plants, _total) =
        db_plants::list_plants_for_user(&app_state.pool, user_id, 1000, 0, None).await?;

    tracing::info!(
        "Found {} plants for user {} when generating calendar",
        plants.len(),
        user_id
    );

    for plant in &plants {
        tracing::info!("Plant: {} - watering: {:?} days, fertilizing: {:?} days, last_watered: {:?}, last_fertilized: {:?}", 
                      plant.name, plant.watering_schedule.interval_days, plant.fertilizing_schedule.interval_days,
                      plant.last_watered, plant.last_fertilized);
    }

    // Get base URL from request headers
    let base_url = get_base_url_from_headers(&headers, &uri);

    // Generate the iCalendar feed
    let calendar_content = generate_plant_calendar(&plants, user_id, &base_url)?;

    tracing::info!(
        "Generated calendar feed for user: {} with {} plants, content length: {} chars",
        user_id,
        plants.len(),
        calendar_content.len()
    );
    tracing::debug!("Calendar content: {}", calendar_content);

    // Return the calendar with proper headers
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/calendar; charset=utf-8")
        .header(header::CACHE_CONTROL, "private, max-age=3600") // Cache for 1 hour
        .header(
            "Content-Disposition",
            &format!("attachment; filename=\"plant-care-{}.ics\"", user_id),
        )
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
    State(app_state): State<AppState>,
    auth_session: AuthSession,
    uri: Uri,
    headers: HeaderMap,
) -> Result<impl IntoResponse> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!("Calendar subscription info request for user: {}", user.id);

    // Calendar token is persisted and reused until explicitly rotated.
    let calendar_token = get_or_create_calendar_token(&app_state.pool, &user.id).await?;

    // Get base URL from request headers or environment
    let base_url =
        std::env::var("BASE_URL").unwrap_or_else(|_| get_base_url_from_headers(&headers, &uri));

    // Determine API prefix from current request URI
    let api_path = if uri.path().starts_with("/api/v1/") {
        "/api/v1/calendar" // Frontend serving mode
    } else {
        "/v1/calendar" // API-only mode
    };
    let feed_url = format!(
        "{}{}/{}.ics?token={}",
        base_url, api_path, user.id, calendar_token
    );

    let response = serde_json::json!({
        "feedUrl": feed_url,
        "tokenLifecycle": {
            "persistent": true,
            "rotation": "Token remains stable until /calendar/regenerate-token is called.",
            "regenerationEffect": "Regenerating invalidates the previous token immediately."
        },
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
    State(app_state): State<AppState>,
    auth_session: AuthSession,
    uri: Uri,
    headers: HeaderMap,
) -> Result<impl IntoResponse> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    tracing::info!("Calendar token regeneration request for user: {}", user.id);

    // Rotate token so old subscriptions are invalidated immediately.
    let calendar_token = rotate_calendar_token(&app_state.pool, &user.id).await?;

    // Get base URL from request headers or environment
    let base_url =
        std::env::var("BASE_URL").unwrap_or_else(|_| get_base_url_from_headers(&headers, &uri));

    // Determine API prefix from current request URI
    let api_path = if uri.path().starts_with("/api/v1/") {
        "/api/v1/calendar" // Frontend serving mode
    } else {
        "/v1/calendar" // API-only mode
    };
    let feed_url = format!(
        "{}{}/{}.ics?token={}",
        base_url, api_path, user.id, calendar_token
    );

    let response = serde_json::json!({
        "feedUrl": feed_url,
        "message": "Calendar token regenerated successfully. The previous token is no longer valid, so update your calendar subscription with this new URL.",
        "tokenLifecycle": {
            "persistent": true,
            "rotation": "Token remains stable until the next regeneration.",
            "regenerationEffect": "Regeneration immediately revokes the previous token."
        }
    });

    Ok(axum::Json(response))
}
