use axum::{
    extract::{Query, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;

use crate::app_state::AppState;
use crate::auth::AuthSession;
use crate::database::invites as db_invites;
use crate::middleware::validation::ValidatedJson;
use crate::models::{
    CreateInviteRequest, InviteResponse, ValidateInviteRequest, WaitlistResponse,
    WaitlistSignupRequest,
};
use crate::utils::errors::{AppError, Result};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/create", post(create_invite))
        .route("/validate", post(validate_invite))
        .route("/list", get(list_invites))
        .route("/waitlist", post(join_waitlist))
        .route("/waitlist/list", get(list_waitlist))
}

#[derive(Deserialize)]
struct ListInvitesQuery {
    created_by: Option<String>,
}

#[utoipa::path(
    post,
    path = "/invites/create",
    request_body = CreateInviteRequest,
    responses(
        (status = 201, description = "Invite code created", body = InviteResponse),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Invalid request"),
    ),
    tag = "invites"
)]
async fn create_invite(
    auth_session: AuthSession,
    ValidatedJson(payload): ValidatedJson<CreateInviteRequest>,
) -> Result<(axum::http::StatusCode, Json<InviteResponse>)> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Authentication required".to_string(),
    })?;

    tracing::info!("Creating invite code for user: {}", user.id);

    let invite = db_invites::create_invite_code(
        &auth_session.backend.db,
        &payload,
        Some(&user.id),
    )
    .await?;

    tracing::info!("Invite code created: {}", invite.code);
    Ok((axum::http::StatusCode::CREATED, Json(invite.into())))
}

#[utoipa::path(
    post,
    path = "/invites/validate",
    request_body = ValidateInviteRequest,
    responses(
        (status = 200, description = "Invite code is valid"),
        (status = 400, description = "Invalid or expired invite code"),
    ),
    tag = "invites"
)]
async fn validate_invite(
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<ValidateInviteRequest>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Validating invite code: {}", payload.code);

    let invite = db_invites::validate_invite_code(&app_state.pool, &payload.code).await?;

    if !invite.is_valid() {
        return Err(AppError::Validation({
            let mut errors = validator::ValidationErrors::new();
            errors.add(
                "code",
                validator::ValidationError::new("invalid_or_expired"),
            );
            errors
        }));
    }

    tracing::info!("Invite code is valid: {}", payload.code);
    Ok(Json(serde_json::json!({
        "valid": true,
        "uses_remaining": invite.max_uses - invite.current_uses
    })))
}

#[utoipa::path(
    get,
    path = "/invites/list",
    params(
        ("created_by" = Option<String>, Query, description = "Filter by creator user ID")
    ),
    responses(
        (status = 200, description = "List of invite codes", body = Vec<InviteResponse>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "invites"
)]
async fn list_invites(
    auth_session: AuthSession,
    Query(params): Query<ListInvitesQuery>,
) -> Result<Json<Vec<InviteResponse>>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Authentication required".to_string(),
    })?;

    tracing::info!("Listing invite codes for user: {}", user.id);

    let created_by = params.created_by.as_deref().or(Some(&user.id));
    let invites = db_invites::list_invite_codes(&auth_session.backend.db, created_by).await?;

    let responses: Vec<InviteResponse> = invites.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

#[utoipa::path(
    post,
    path = "/invites/waitlist",
    request_body = WaitlistSignupRequest,
    responses(
        (status = 201, description = "Added to waitlist", body = WaitlistResponse),
        (status = 400, description = "Invalid request or email already exists"),
    ),
    tag = "invites"
)]
async fn join_waitlist(
    auth_session: AuthSession,
    ValidatedJson(payload): ValidatedJson<WaitlistSignupRequest>,
) -> Result<(axum::http::StatusCode, Json<WaitlistResponse>)> {
    tracing::info!("Adding to waitlist: {}", payload.email);

    let entry = db_invites::add_to_waitlist(&auth_session.backend.db, &payload).await?;

    tracing::info!("Added to waitlist: {}", payload.email);
    Ok((axum::http::StatusCode::CREATED, Json(entry.into())))
}

#[utoipa::path(
    get,
    path = "/invites/waitlist/list",
    responses(
        (status = 200, description = "List of waitlist entries", body = Vec<WaitlistResponse>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "invites"
)]
async fn list_waitlist(
    auth_session: AuthSession,
) -> Result<Json<Vec<WaitlistResponse>>> {
    let _user = auth_session.user.ok_or(AppError::Authentication {
        message: "Authentication required".to_string(),
    })?;

    tracing::info!("Listing waitlist entries");

    let entries = db_invites::get_waitlist_entries(&auth_session.backend.db).await?;

    let responses: Vec<WaitlistResponse> = entries.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}