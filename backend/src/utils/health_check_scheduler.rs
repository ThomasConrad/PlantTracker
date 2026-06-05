//! Daily health check scheduler.
//!
//! Runs once daily (at ~3 AM local or after startup), iterating all plants
//! with photos and running an AI health assessment on each.

use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::database::DatabasePool;
use crate::handlers::memory::{run_health_assessment, HealthAssessmentResult};
use crate::llm::PlantCoach;
use crate::utils::push::{PushCategory, PushPayload, send_push_if_allowed};

/// Start the daily health check as a background task.
/// Returns immediately; the actual work runs in a spawned task.
pub fn start_health_check_scheduler(app_state: AppState) {
    tokio::spawn(async move {
        // Wait 60 seconds after startup before the first run
        // (let the server finish initializing, migrations, etc.)
        info!("Health check scheduler: waiting 60s before first run");
        sleep(Duration::from_secs(60)).await;

        loop {
            info!("Health check scheduler: starting daily assessment run");

            match run_all_assessments(&app_state).await {
                Ok((assessed, failed, total)) => {
                    info!(
                        "Health check scheduler complete: assessed={}, failed={}, total={}",
                        assessed, failed, total
                    );
                }
                Err(e) => {
                    error!("Health check scheduler error: {}", e);
                }
            }

            // Sleep for 24 hours before next run
            info!("Health check scheduler: sleeping for 24 hours");
            sleep(Duration::from_secs(24 * 60 * 60)).await;
        }
    });
}

/// Run health assessments for all plants that have photos.
/// Returns (assessed, failed, total).
async fn run_all_assessments(app_state: &AppState) -> Result<(u32, u32, usize), String> {
    // Get all plant IDs that have at least one photo and are not archived
    let plants_with_photos = get_plants_needing_assessment(&app_state.pool)
        .await
        .map_err(|e| format!("Failed to query plants: {e}"))?;

    let total = plants_with_photos.len();
    let mut assessed = 0u32;
    let mut failed = 0u32;

    // Collect alerts per-user so we can send a single summary push per user
    let mut user_alerts: std::collections::HashMap<String, Vec<HealthAssessmentResult>> =
        std::collections::HashMap::new();

    for (plant_id_str, user_id) in &plants_with_photos {
        let plant_id = match Uuid::parse_str(plant_id_str) {
            Ok(id) => id,
            Err(_) => {
                failed += 1;
                continue;
            }
        };

        // Get the user's LLM settings to build a coach
        let plant_user = match crate::database::users::get_user_by_id(&app_state.pool, user_id).await {
            Ok(u) => u,
            Err(_) => {
                failed += 1;
                continue;
            }
        };

        // Resolve the coach for this user
        let coach: Arc<dyn PlantCoach> = match crate::llm::resolve_coach_for_request(
            plant_user.llm_base_url.as_deref(),
            plant_user.llm_api_key.as_deref(),
            plant_user.llm_model.as_deref(),
            app_state.coach.as_ref(),
        ) {
            Ok(c) => c,
            Err(_) => {
                // No coach available for this user, skip
                failed += 1;
                continue;
            }
        };

        // Run the assessment
        match run_health_assessment(app_state, &plant_id, user_id, coach.as_ref()).await {
            Ok(result) => {
                assessed += 1;
                // Track plants with concerning health (score <= 2/5)
                if result.raw_ai_score <= 2 {
                    user_alerts
                        .entry(user_id.clone())
                        .or_default()
                        .push(result);
                }
            }
            Err(e) => {
                warn!(
                    "Health check failed for plant {}: {}",
                    plant_id, e
                );
                failed += 1;
            }
        }

        // Brief pause between plants to avoid API rate limits
        sleep(Duration::from_millis(1000)).await;
    }

    // Send push notifications for users with concerning plants
    send_daily_health_alerts(app_state, &user_alerts).await;

    Ok((assessed, failed, total))
}

/// Send push notifications summarizing plants that need attention.
async fn send_daily_health_alerts(
    app_state: &AppState,
    user_alerts: &std::collections::HashMap<String, Vec<HealthAssessmentResult>>,
) {
    if user_alerts.is_empty() {
        return;
    }

    for (user_id, alerts) in user_alerts {
        let (title, body) = if alerts.len() == 1 {
            let a = &alerts[0];
            (
                format!("{} needs attention", a.plant_name),
                format!("Health score: {}/5", a.raw_ai_score),
            )
        } else {
            let names: Vec<&str> = alerts.iter().take(3).map(|a| a.plant_name.as_str()).collect();
            let suffix = if alerts.len() > 3 {
                format!(" and {} more", alerts.len() - 3)
            } else {
                String::new()
            };
            (
                format!("{} plants need attention", alerts.len()),
                format!("{}{}", names.join(", "), suffix),
            )
        };

        let payload = PushPayload {
            title,
            body,
            url: Some("/plants".to_string()),
            icon: None,
            tag: Some("daily-health-summary".to_string()),
        };

        send_push_if_allowed(&app_state.pool, user_id, PushCategory::DailySummary, &payload).await;
    }
}

/// Get all (plant_id, user_id) pairs for plants that have photos and are not archived.
/// Only includes plants that haven't been assessed today.
async fn get_plants_needing_assessment(
    pool: &DatabasePool,
) -> Result<Vec<(String, String)>, sqlx::Error> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let rows = sqlx::query!(
        r#"SELECT DISTINCT p.id as "plant_id!: String", p.user_id as "user_id!: String"
         FROM plants p
         JOIN photos ph ON ph.plant_id = p.id
         WHERE p.archived_at IS NULL
         AND NOT EXISTS (
             SELECT 1 FROM plant_health_scores hs
             WHERE hs.plant_id = p.id AND hs.scored_at = ?
         )
         ORDER BY p.updated_at DESC"#,
        today
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| (r.plant_id, r.user_id)).collect())
}
