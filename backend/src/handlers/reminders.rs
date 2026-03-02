use axum::{
    response::Json,
    routing::{get, post},
    Router,
};

use crate::app_state::AppState;
use crate::auth::AuthSession;
use crate::database::reminders as db_reminders;
use crate::middleware::validation::ValidatedJson;
use crate::models::{
    DispatchRemindersResponse, DueRemindersResponse, ReminderPreferences,
    UpdateReminderPreferencesRequest,
};
use crate::utils::errors::{AppError, Result};

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
async fn get_preferences(auth_session: AuthSession) -> Result<Json<ReminderPreferences>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Authentication required".to_string(),
    })?;

    let prefs = db_reminders::get_or_create_preferences(&auth_session.backend.db, &user.id).await?;
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
    auth_session: AuthSession,
    ValidatedJson(payload): ValidatedJson<UpdateReminderPreferencesRequest>,
) -> Result<Json<ReminderPreferences>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Authentication required".to_string(),
    })?;

    let prefs =
        db_reminders::update_preferences(&auth_session.backend.db, &user.id, &payload).await?;
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
async fn get_due_reminders(auth_session: AuthSession) -> Result<Json<DueRemindersResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Authentication required".to_string(),
    })?;

    let reminders = db_reminders::list_due_reminders(&auth_session.backend.db, &user.id).await?;
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
    auth_session: AuthSession,
) -> Result<Json<DispatchRemindersResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Authentication required".to_string(),
    })?;

    let prefs = db_reminders::get_or_create_preferences(&auth_session.backend.db, &user.id).await?;
    if !prefs.enabled {
        return Ok(Json(DispatchRemindersResponse {
            reminders: vec![],
            sent_count: 0,
        }));
    }

    let reminders =
        db_reminders::dispatch_due_reminders(&auth_session.backend.db, &user.id).await?;
    Ok(Json(DispatchRemindersResponse {
        sent_count: reminders.len(),
        reminders,
    }))
}
