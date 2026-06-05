use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};

use crate::app_state::AppState;
use crate::database::reminders as db_reminders;
use crate::extractors::AuthenticatedUser;
use crate::middleware::validation::ValidatedJson;
use crate::models::{
    DispatchRemindersResponse, DueRemindersResponse, ReminderPreferences,
    UpdateReminderPreferencesRequest,
};
use crate::utils::errors::Result;
use crate::utils::push::{PushCategory, PushPayload, send_push_if_allowed};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/preferences", get(get_preferences).put(update_preferences))
        .route("/due", get(get_due_reminders))
        .route("/dispatch", post(dispatch_due_reminders))
}

#[utoipa::path(
    get,
    path = "/reminders/preferences",
    responses(
        (status = 200, description = "Reminder preferences", body = ReminderPreferences),
        (status = 401, description = "Unauthorized")
    ),
    security(("session" = [])),
    tag = "reminders"
)]
async fn get_preferences(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
) -> Result<Json<ReminderPreferences>> {
    let prefs = db_reminders::get_or_create_preferences(&app_state.pool, &user.id).await?;
    Ok(Json(prefs))
}

#[utoipa::path(
    put,
    path = "/reminders/preferences",
    request_body = UpdateReminderPreferencesRequest,
    responses(
        (status = 200, description = "Updated reminder preferences", body = ReminderPreferences),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation error")
    ),
    security(("session" = [])),
    tag = "reminders"
)]
async fn update_preferences(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<UpdateReminderPreferencesRequest>,
) -> Result<Json<ReminderPreferences>> {
    let prefs =
        db_reminders::update_preferences(&app_state.pool, &user.id, &payload).await?;
    Ok(Json(prefs))
}

#[utoipa::path(
    get,
    path = "/reminders/due",
    responses(
        (status = 200, description = "Due reminders list", body = DueRemindersResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("session" = [])),
    tag = "reminders"
)]
async fn get_due_reminders(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
) -> Result<Json<DueRemindersResponse>> {
    let reminders = db_reminders::list_due_reminders(&app_state.pool, &user.id).await?;
    let unsent_count = reminders.iter().filter(|r| !r.already_sent).count();

    Ok(Json(DueRemindersResponse {
        total_due: reminders.len(),
        unsent_count,
        reminders,
    }))
}

#[utoipa::path(
    post,
    path = "/reminders/dispatch",
    responses(
        (status = 200, description = "Dispatched due reminders", body = DispatchRemindersResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("session" = [])),
    tag = "reminders"
)]
async fn dispatch_due_reminders(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
) -> Result<Json<DispatchRemindersResponse>> {
    let prefs = db_reminders::get_or_create_preferences(&app_state.pool, &user.id).await?;
    if !prefs.enabled {
        return Ok(Json(DispatchRemindersResponse {
            reminders: vec![],
            sent_count: 0,
        }));
    }

    let pool = app_state.pool.clone();
    let reminders =
        db_reminders::dispatch_due_reminders(&pool, &user.id).await?;

    // Send push notifications for dispatched reminders (fire-and-forget)
    if !reminders.is_empty() {
        let user_id = user.id.clone();
        let reminder_count = reminders.len();
        let reminders_for_push = reminders.clone();
        tokio::spawn(async move {
            let (title, body) = if reminder_count == 1 {
                let r = &reminders_for_push[0];
                let action = if r.reminder_type == "watering" { "Water" } else { "Care for" };
                (
                    format!("{} {}", action, r.plant_name),
                    if r.days_overdue > 0 {
                        format!("{} day(s) overdue", r.days_overdue)
                    } else {
                        "Due today".to_string()
                    },
                )
            } else {
                let names: Vec<&str> = reminders_for_push.iter().take(3).map(|r| r.plant_name.as_str()).collect();
                let suffix = if reminder_count > 3 {
                    format!(" and {} more", reminder_count - 3)
                } else {
                    String::new()
                };
                (
                    format!("{} plants need care", reminder_count),
                    format!("{}{}", names.join(", "), suffix),
                )
            };

            let payload = PushPayload {
                title,
                body,
                url: Some("/reminders".to_string()),
                icon: None,
                tag: Some("reminder-dispatch".to_string()),
            };

            send_push_if_allowed(&pool, &user_id, PushCategory::Reminder, &payload).await;
        });
    }

    Ok(Json(DispatchRemindersResponse {
        sent_count: reminders.len(),
        reminders,
    }))
}
