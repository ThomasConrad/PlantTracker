use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::app_state::AppState;
use crate::database::invites as db_invites;
use crate::extractors::AuthenticatedUser;
use crate::middleware::validation::ValidatedJson;
use crate::models::{
    CreateInviteRequest, InviteResponse, ValidateInviteRequest, WaitlistResponse,
    WaitlistSignupRequest,
};
use crate::utils::errors::{AppError, Result};

pub fn routes() -> Router<AppState> {
    let mut router = Router::new()
        .route("/create", post(create_invite))
        .route("/validate", post(validate_invite))
        .route("/list", get(list_invites));

    if waitlist_enabled() {
        router = router
            .route("/waitlist", post(join_waitlist))
            .route("/waitlist/list", get(list_waitlist))
            .route("/waitlist/:waitlist_id/invite", post(invite_waitlist_entry));
    }

    router
}

fn waitlist_enabled() -> bool {
    std::env::var("WAITLIST_ENABLED")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

#[derive(Deserialize)]
struct ListInvitesQuery {
    created_by: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
struct InviteWaitlistRequest {
    max_uses: Option<i32>,
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
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateInviteRequest>,
) -> Result<(axum::http::StatusCode, Json<InviteResponse>)> {

    if !user.can_create_invite() {
        return Err(AppError::Authorization {
            message:
                "You do not have invite creation permissions or have reached your invite limit"
                    .to_string(),
        });
    }

    tracing::info!("Creating invite code for user: {}", user.id);

    let invite = db_invites::create_invite_code_consuming_quota(
        &app_state.pool,
        &payload,
        &user.id,
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
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    Query(params): Query<ListInvitesQuery>,
) -> Result<Json<serde_json::Value>> {

    tracing::info!("Listing invite codes for user: {}", user.id);

    let created_by = params.created_by.as_deref().or(Some(&user.id));
    let invites = db_invites::list_invite_codes(&app_state.pool, created_by).await?;

    let responses: Vec<InviteResponse> = invites.into_iter().map(Into::into).collect();
    Ok(Json(serde_json::json!({
        "invites": responses
    })))
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
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<WaitlistSignupRequest>,
) -> Result<(axum::http::StatusCode, Json<WaitlistResponse>)> {
    if !waitlist_enabled() {
        return Err(AppError::NotFound {
            resource: "Waitlist".to_string(),
        });
    }

    tracing::info!("Adding to waitlist: {}", payload.email);

    let entry = db_invites::add_to_waitlist(&app_state.pool, &payload).await?;

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
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
) -> Result<Json<Vec<WaitlistResponse>>> {
    if !waitlist_enabled() {
        return Err(AppError::NotFound {
            resource: "Waitlist".to_string(),
        });
    }

    if !user.is_admin() {
        return Err(AppError::Authorization {
            message: "Admin access required".to_string(),
        });
    }

    tracing::info!("Listing waitlist entries");

    let entries = db_invites::get_waitlist_entries(&app_state.pool).await?;

    let responses: Vec<WaitlistResponse> = entries.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

#[utoipa::path(
    post,
    path = "/invites/waitlist/{waitlist_id}/invite",
    params(
        ("waitlist_id" = String, Path, description = "Waitlist entry ID")
    ),
    request_body = InviteWaitlistRequest,
    responses(
        (status = 200, description = "Invite created and waitlist entry updated"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "Waitlist entry not found")
    ),
    tag = "invites"
)]
async fn invite_waitlist_entry(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    Path(waitlist_id): Path<String>,
    Json(payload): Json<InviteWaitlistRequest>,
) -> Result<Json<serde_json::Value>> {
    if !waitlist_enabled() {
        return Err(AppError::NotFound {
            resource: "Waitlist".to_string(),
        });
    }

    if !user.is_admin() {
        return Err(AppError::Authorization {
            message: "Admin access required".to_string(),
        });
    }

    let max_uses = payload.max_uses.unwrap_or(1).max(1);
    let (entry, invite) = db_invites::invite_waitlist_entry(
        &app_state.pool,
        &waitlist_id,
        &user.id,
        max_uses,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "invite_code": invite.code,
        "waitlist_entry": WaitlistResponse::from(entry)
    })))
}
