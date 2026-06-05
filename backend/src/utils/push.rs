//! Web Push notification utilities.
//!
//! Handles sending push notifications via the Web Push protocol (RFC 8030)
//! with VAPID authentication.

use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use web_push::{
    ContentEncoding, IsahcWebPushClient, SubscriptionInfo, VapidSignatureBuilder, WebPushClient,
    WebPushMessageBuilder,
};

use crate::database::DatabasePool;

/// Push notification configuration (from environment)
#[derive(Debug, Clone)]
pub struct PushConfig {
    pub public_key: String,
    pub private_key: String,
    pub subject: String,
}

impl PushConfig {
    /// Load push configuration from environment variables
    pub fn from_env() -> Option<Self> {
        let public_key = std::env::var("VAPID_PUBLIC_KEY").ok()?;
        let private_key = std::env::var("VAPID_PRIVATE_KEY").ok()?;
        let subject = std::env::var("VAPID_SUBJECT")
            .unwrap_or_else(|_| "mailto:admin@example.com".to_string());

        if public_key.is_empty() || private_key.is_empty() {
            return None;
        }

        Some(Self {
            public_key,
            private_key,
            subject,
        })
    }
}

/// Payload sent in the push notification
#[derive(Debug, Serialize, Deserialize)]
pub struct PushPayload {
    /// Title shown in the notification
    pub title: String,
    /// Body text
    pub body: String,
    /// URL to navigate to when clicked
    pub url: Option<String>,
    /// Icon URL (falls back to app icon)
    pub icon: Option<String>,
    /// Tag for notification grouping/replacement
    pub tag: Option<String>,
}

/// Send a push notification to all subscriptions for a user.
/// Returns the number of successful deliveries.
pub async fn send_push_to_user(
    pool: &DatabasePool,
    config: &PushConfig,
    user_id: &str,
    payload: &PushPayload,
) -> u32 {
    let subscriptions = match get_user_subscriptions(pool, user_id).await {
        Ok(subs) => subs,
        Err(e) => {
            error!(
                "Failed to get push subscriptions for user {}: {}",
                user_id, e
            );
            return 0;
        }
    };

    if subscriptions.is_empty() {
        return 0;
    }

    let payload_json = match serde_json::to_string(payload) {
        Ok(j) => j,
        Err(e) => {
            error!("Failed to serialize push payload: {}", e);
            return 0;
        }
    };

    let client = match IsahcWebPushClient::new() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to create push client: {}", e);
            return 0;
        }
    };

    let mut successes = 0u32;

    for sub in &subscriptions {
        match send_single_push(config, &client, sub, &payload_json).await {
            Ok(()) => successes += 1,
            Err(e) => {
                warn!(
                    "Push notification failed for subscription {}: {}",
                    sub.id, e
                );
                // If the subscription is gone (410 Gone), remove it
                if e.contains("410") || e.contains("Gone") || e.contains("expired") {
                    info!("Removing expired push subscription {}", sub.id);
                    let _ = delete_subscription(pool, &sub.id).await;
                }
            }
        }
    }

    successes
}

/// Push notification category (used to check user preferences)
#[derive(Debug, Clone, Copy)]
pub enum PushCategory {
    HealthAlert,
    DailySummary,
    CoachSuggestion,
    Reminder,
}

/// Send a push notification if the user has opted in for this category.
/// Returns the number of successful deliveries (0 if user opted out or no subscriptions).
pub async fn send_push_if_allowed(
    pool: &DatabasePool,
    user_id: &str,
    category: PushCategory,
    payload: &PushPayload,
) -> u32 {
    let config = match PushConfig::from_env() {
        Some(c) => c,
        None => return 0,
    };

    // Check user preferences for this category
    let allowed = match check_push_preference(pool, user_id, category).await {
        Ok(allowed) => allowed,
        Err(e) => {
            warn!(
                "Failed to check push preferences for user {}: {}",
                user_id, e
            );
            // Default to sending if we can't check prefs
            true
        }
    };

    if !allowed {
        return 0;
    }

    send_push_to_user(pool, &config, user_id, payload).await
}

/// Check if a user has opted in for a specific push category
async fn check_push_preference(
    pool: &DatabasePool,
    user_id: &str,
    category: PushCategory,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT push_health_alerts, push_daily_summary, push_coach_suggestions, push_reminders
         FROM user_reminder_preferences WHERE user_id = ?"#,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        // No preferences row = defaults (all enabled)
        return Ok(true);
    };

    Ok(match category {
        PushCategory::HealthAlert => row.push_health_alerts,
        PushCategory::DailySummary => row.push_daily_summary,
        PushCategory::CoachSuggestion => row.push_coach_suggestions,
        PushCategory::Reminder => row.push_reminders,
    })
}

/// Send a push notification to a single subscription
async fn send_single_push(
    config: &PushConfig,
    client: &IsahcWebPushClient,
    sub: &PushSubscriptionRow,
    payload_json: &str,
) -> Result<(), String> {
    let subscription_info = SubscriptionInfo::new(&sub.endpoint, &sub.p256dh, &sub.auth);

    let mut sig_builder =
        VapidSignatureBuilder::from_base64(&config.private_key, &subscription_info)
            .map_err(|e| format!("VAPID signature error: {e}"))?;

    sig_builder.add_claim("sub", config.subject.clone());

    let vapid_signature = sig_builder
        .build()
        .map_err(|e| format!("VAPID build error: {e}"))?;

    let mut builder = WebPushMessageBuilder::new(&subscription_info);
    builder.set_payload(ContentEncoding::Aes128Gcm, payload_json.as_bytes());
    builder.set_vapid_signature(vapid_signature);

    let message = builder
        .build()
        .map_err(|e| format!("Message build error: {e}"))?;

    client.send(message).await.map_err(|e| format!("{e}"))?;

    Ok(())
}

// ─── Database helpers ───────────────────────────────────────────────────────

#[derive(Debug)]
struct PushSubscriptionRow {
    id: String,
    endpoint: String,
    p256dh: String,
    auth: String,
}

async fn get_user_subscriptions(
    pool: &DatabasePool,
    user_id: &str,
) -> Result<Vec<PushSubscriptionRow>, sqlx::Error> {
    let rows = sqlx::query_as!(
        PushSubscriptionRow,
        r#"SELECT id as "id!: String", endpoint as "endpoint!: String", p256dh as "p256dh!: String", auth as "auth!: String"
         FROM push_subscriptions WHERE user_id = ?"#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

async fn delete_subscription(
    pool: &DatabasePool,
    subscription_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM push_subscriptions WHERE id = ?",
        subscription_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Save a new push subscription for a user (upsert by endpoint)
pub async fn save_subscription(
    pool: &DatabasePool,
    user_id: &str,
    endpoint: &str,
    p256dh: &str,
    auth: &str,
    user_agent: Option<&str>,
) -> Result<String, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Upsert: if this endpoint already exists, update it
    sqlx::query!(
        r#"INSERT INTO push_subscriptions (id, user_id, endpoint, p256dh, auth, user_agent, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(endpoint) DO UPDATE SET
             p256dh = excluded.p256dh,
             auth = excluded.auth,
             user_agent = excluded.user_agent,
             updated_at = excluded.updated_at"#,
        id,
        user_id,
        endpoint,
        p256dh,
        auth,
        user_agent,
        now,
        now
    )
    .execute(pool)
    .await?;

    Ok(id)
}

/// Remove a push subscription by endpoint
pub async fn remove_subscription_by_endpoint(
    pool: &DatabasePool,
    user_id: &str,
    endpoint: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM push_subscriptions WHERE user_id = ? AND endpoint = ?",
        user_id,
        endpoint
    )
    .execute(pool)
    .await?;
    Ok(())
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // These tests modify process-wide env vars, so they must not run in parallel.
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn push_config_from_env_returns_none_when_missing() {
        let _lock = ENV_MUTEX.lock().unwrap();
        std::env::remove_var("VAPID_PUBLIC_KEY");
        std::env::remove_var("VAPID_PRIVATE_KEY");

        let config = PushConfig::from_env();
        assert!(config.is_none());
    }

    #[test]
    fn push_config_from_env_returns_none_when_empty() {
        let _lock = ENV_MUTEX.lock().unwrap();
        std::env::set_var("VAPID_PUBLIC_KEY", "");
        std::env::set_var("VAPID_PRIVATE_KEY", "");

        let config = PushConfig::from_env();
        assert!(config.is_none());

        // Clean up
        std::env::remove_var("VAPID_PUBLIC_KEY");
        std::env::remove_var("VAPID_PRIVATE_KEY");
    }

    #[test]
    fn push_config_from_env_parses_correctly() {
        let _lock = ENV_MUTEX.lock().unwrap();
        std::env::set_var("VAPID_PUBLIC_KEY", "test-public-key");
        std::env::set_var("VAPID_PRIVATE_KEY", "test-private-key");
        std::env::set_var("VAPID_SUBJECT", "mailto:test@example.com");

        let config = PushConfig::from_env().unwrap();
        assert_eq!(config.public_key, "test-public-key");
        assert_eq!(config.private_key, "test-private-key");
        assert_eq!(config.subject, "mailto:test@example.com");

        // Clean up
        std::env::remove_var("VAPID_PUBLIC_KEY");
        std::env::remove_var("VAPID_PRIVATE_KEY");
        std::env::remove_var("VAPID_SUBJECT");
    }

    #[test]
    fn push_config_defaults_subject_when_not_set() {
        let _lock = ENV_MUTEX.lock().unwrap();
        std::env::set_var("VAPID_PUBLIC_KEY", "pk");
        std::env::set_var("VAPID_PRIVATE_KEY", "sk");
        std::env::remove_var("VAPID_SUBJECT");

        let config = PushConfig::from_env().unwrap();
        assert_eq!(config.subject, "mailto:admin@example.com");

        // Clean up
        std::env::remove_var("VAPID_PUBLIC_KEY");
        std::env::remove_var("VAPID_PRIVATE_KEY");
    }

    #[test]
    fn push_payload_serializes_correctly() {
        let payload = PushPayload {
            title: "Water your Fern".to_string(),
            body: "It's looking thirsty!".to_string(),
            url: Some("/plants/123".to_string()),
            icon: None,
            tag: Some("health-alert-fern".to_string()),
        };

        let json = serde_json::to_string(&payload).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["title"], "Water your Fern");
        assert_eq!(parsed["body"], "It's looking thirsty!");
        assert_eq!(parsed["url"], "/plants/123");
        assert_eq!(parsed["icon"], serde_json::Value::Null);
        assert_eq!(parsed["tag"], "health-alert-fern");
    }

    #[test]
    fn push_payload_deserializes_with_optional_fields_missing() {
        let json = r#"{"title":"Test","body":"Hello"}"#;
        let payload: PushPayload = serde_json::from_str(json).unwrap();

        assert_eq!(payload.title, "Test");
        assert_eq!(payload.body, "Hello");
        assert!(payload.url.is_none());
        assert!(payload.icon.is_none());
        assert!(payload.tag.is_none());
    }
}
