//! Push notification subscription endpoints.

use axum::{
    extract::State,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;

use crate::app_state::AppState;
use crate::extractors::AuthenticatedUser;
use crate::utils::errors::{AppError, Result};
use crate::utils::push;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/vapid-key", get(get_vapid_public_key))
        .route("/subscribe", post(subscribe))
        .route("/unsubscribe", delete(unsubscribe))
        .route("/test", post(test_push))
}

// ─── Types ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VapidKeyResponse {
    pub public_key: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubscribeRequest {
    pub endpoint: String,
    pub keys: SubscriptionKeys,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UnsubscribeRequest {
    pub endpoint: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SubscribeResponse {
    pub ok: bool,
}

// ─── Handlers ───────────────────────────────────────────────────────────────

/// Get the VAPID public key for the client to use when subscribing.
#[utoipa::path(
    get,
    path = "/push/vapid-key",
    responses(
        (status = 200, description = "VAPID public key", body = VapidKeyResponse),
        (status = 503, description = "Push notifications not configured"),
    ),
    tag = "push"
)]
async fn get_vapid_public_key(
    State(_app_state): State<AppState>,
) -> Result<Json<VapidKeyResponse>> {
    let config = push::PushConfig::from_env().ok_or(AppError::External {
        message: "Push notifications are not configured on this server".to_string(),
    })?;

    Ok(Json(VapidKeyResponse {
        public_key: config.public_key,
    }))
}

/// Subscribe to push notifications.
/// Stores the browser's push subscription for the authenticated user.
#[utoipa::path(
    post,
    path = "/push/subscribe",
    request_body = SubscribeRequest,
    responses(
        (status = 200, description = "Subscription saved", body = SubscribeResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(("session" = [])),
    tag = "push"
)]
async fn subscribe(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    Json(payload): Json<SubscribeRequest>,
) -> Result<Json<SubscribeResponse>> {
    push::save_subscription(
        &app_state.pool,
        &user.id,
        &payload.endpoint,
        &payload.keys.p256dh,
        &payload.keys.auth,
        None,
    )
    .await
    .map_err(|e| AppError::Internal {
        message: format!("Failed to save push subscription: {e}"),
    })?;

    info!("Push subscription saved for user {}", user.id);
    Ok(Json(SubscribeResponse { ok: true }))
}

/// Unsubscribe from push notifications.
/// Removes the subscription matching the given endpoint.
#[utoipa::path(
    delete,
    path = "/push/unsubscribe",
    request_body = UnsubscribeRequest,
    responses(
        (status = 200, description = "Subscription removed", body = SubscribeResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(("session" = [])),
    tag = "push"
)]
async fn unsubscribe(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    Json(payload): Json<UnsubscribeRequest>,
) -> Result<Json<SubscribeResponse>> {
    push::remove_subscription_by_endpoint(&app_state.pool, &user.id, &payload.endpoint)
        .await
        .map_err(|e| AppError::Internal {
            message: format!("Failed to remove push subscription: {e}"),
        })?;

    info!("Push subscription removed for user {}", user.id);
    Ok(Json(SubscribeResponse { ok: true }))
}

/// Send a test push notification to verify the subscription works.
#[utoipa::path(
    post,
    path = "/push/test",
    responses(
        (status = 200, description = "Test notification sent", body = TestPushResponse),
        (status = 401, description = "Unauthorized"),
        (status = 503, description = "Push not configured or no subscriptions"),
    ),
    security(("session" = [])),
    tag = "push"
)]
async fn test_push(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
) -> Result<Json<TestPushResponse>> {
    let config = push::PushConfig::from_env().ok_or(AppError::External {
        message: "Push notifications are not configured on this server".to_string(),
    })?;

    let payload = push::PushPayload {
        title: "Planty Test".to_string(),
        body: "Push notifications are working! You'll receive alerts about your plants here."
            .to_string(),
        url: Some("/settings".to_string()),
        icon: None,
        tag: Some("test-notification".to_string()),
    };

    let sent = push::send_push_to_user(&app_state.pool, &config, &user.id, &payload).await;

    if sent == 0 {
        return Err(AppError::External {
            message: "No active push subscriptions found. Make sure notifications are enabled."
                .to_string(),
        });
    }

    info!(
        "Sent test push notification to user {} ({} device(s))",
        user.id, sent
    );
    Ok(Json(TestPushResponse { sent }))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TestPushResponse {
    pub sent: u32,
}
