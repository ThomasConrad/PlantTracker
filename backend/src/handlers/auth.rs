use axum::{
    body::Body,
    extract::{Multipart, State},
    http::{header, StatusCode},
    response::Json,
    response::Response,
    routing::{delete, get, post, put},
    Router,
};

use crate::app_state::AppState;
use crate::auth::{AuthSession, Credentials};
use crate::database::users as db_users;
use crate::extractors::AuthenticatedUser;
use crate::middleware::validation::ValidatedJson;
use crate::models::{
    AuthResponse, ChangePasswordRequest, CreateUserRequest, DeleteAccountRequest, LoginRequest,
    UpdateLlmSettingsRequest, UpdateProfileRequest, UserResponse, UserRole,
};
use crate::utils::errors::{AppError, Result};
use crate::utils::image_processing::process_uploaded_image;

const DEFAULT_ADMIN_INVITE_LIMIT: i32 = 50;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/profile", put(update_profile))
        .route(
            "/profile-picture",
            post(upload_profile_picture)
                .get(get_profile_picture)
                .delete(delete_profile_picture),
        )
        .route("/change-password", post(change_password))
        .route("/llm-settings", put(update_llm_settings))
        .route("/export", get(export_data))
        .route("/account", delete(delete_account))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 400, description = "Invalid credentials"),
        (status = 401, description = "Authentication failed"),
    )
)]
async fn login(
    mut auth_session: AuthSession,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    tracing::info!("Login attempt for email: {}", payload.email);

    let credentials = Credentials {
        email: payload.email.clone(),
        password: payload.password,
    };

    let user = match auth_session.authenticate(credentials).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::warn!("Failed login attempt for email: {}", payload.email);
            return Err(AppError::Authentication {
                message: "Invalid email or password".to_string(),
            });
        }
        Err(e) => {
            tracing::error!("Authentication error for email {}: {}", payload.email, e);
            return Err(AppError::Internal {
                message: "Authentication system error".to_string(),
            });
        }
    };

    if let Err(e) = auth_session.login(&user).await {
        tracing::error!("Failed to create session for user {}: {}", user.id, e);
        return Err(AppError::Internal {
            message: "Failed to create session".to_string(),
        });
    }

    let response = AuthResponse { user: user.into() };

    tracing::info!("Login successful for email: {}", payload.email);
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "Registration successful", body = AuthResponse),
        (status = 400, description = "Invalid registration data"),
        (status = 409, description = "Email already exists"),
    )
)]
async fn register(
    mut auth_session: AuthSession,
    ValidatedJson(payload): ValidatedJson<CreateUserRequest>,
) -> Result<(axum::http::StatusCode, Json<AuthResponse>)> {
    tracing::info!("Registration attempt for email: {}", payload.email);

    // Validate invite code if provided
    let (is_admin_invite, invite_code) = if let Some(invite_code) = &payload.invite_code {
        use crate::database::invites as db_invites;

        let invite = db_invites::validate_invite_code(&auth_session.backend.db, invite_code)
            .await
            .map_err(|_| AppError::Authentication {
                message: "Invalid or expired invite code".to_string(),
            })?;

        if !invite.is_valid() {
            return Err(AppError::Authentication {
                message: "Invalid or expired invite code".to_string(),
            });
        }

        // Check if this is an admin invite code
        let is_admin = invite_code.starts_with("ADMIN-");
        (is_admin, invite_code.clone())
    } else {
        // No invite code provided - check if open registration is enabled
        let open_reg = std::env::var("PLANTY_OPEN_REGISTRATION")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        if open_reg {
            (false, String::new())
        } else {
            return Err(AppError::Authentication {
                message: "Registration requires a valid invite code".to_string(),
            });
        }
    };

    // Create user in database with appropriate role
    let user = if is_admin_invite {
        // Create admin user with a large but finite invite quota.
        db_users::create_user_internal(
            &auth_session.backend.db,
            &payload,
            UserRole::Admin,
            true,
            Some(DEFAULT_ADMIN_INVITE_LIMIT),
        )
        .await
    } else {
        // Create regular user
        db_users::create_user(&auth_session.backend.db, &payload).await
    }?;

    // Mark invite code as used (skip if open registration with no code)
    use crate::database::invites as db_invites;

    if !invite_code.is_empty() {
        if let Err(e) =
            db_invites::use_invite_code(&auth_session.backend.db, &invite_code, &user.id).await
        {
            tracing::error!("Failed to mark invite code as used: {}", e);
        }
        // Don't fail registration if we can't update invite code
    }

    // Update waitlist status if user was on waitlist
    if let Err(e) = db_invites::update_waitlist_status(
        &auth_session.backend.db,
        &payload.email,
        "registered",
        Some(&invite_code),
    )
    .await
    {
        tracing::debug!("User was not on waitlist or failed to update status: {}", e);
        // This is fine - user might not have been on waitlist
    }

    // Log admin user creation
    if is_admin_invite {
        tracing::info!("🎉 Admin user created: {} ({})", payload.email, user.id);
        println!("🎉 Admin user successfully created: {}", payload.email);
    }

    // Log the user in immediately after registration
    if let Err(e) = auth_session.login(&user).await {
        tracing::error!("Failed to create session for new user {}: {}", user.id, e);
        return Err(AppError::Internal {
            message: "Failed to create session".to_string(),
        });
    }

    let response = AuthResponse { user: user.into() };

    tracing::info!("Registration successful for email: {}", payload.email);
    Ok((axum::http::StatusCode::CREATED, Json(response)))
}

async fn me(AuthenticatedUser(user): AuthenticatedUser) -> Result<Json<UserResponse>> {
    tracing::debug!("Retrieved user profile: {}", user.email);
    Ok(Json(user.into()))
}

async fn logout(mut auth_session: AuthSession) -> Result<axum::http::StatusCode> {
    match auth_session.logout().await {
        Ok(_) => {
            tracing::info!("User logged out successfully");
            Ok(axum::http::StatusCode::OK)
        }
        Err(e) => {
            tracing::error!("Failed to logout: {}", e);
            Err(AppError::Internal {
                message: "Failed to logout".to_string(),
            })
        }
    }
}

#[utoipa::path(
    put,
    path = "/auth/profile",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated", body = UserResponse),
        (status = 401, description = "Unauthorized"),
        (status = 422, description = "Validation error"),
    ),
    security(("session" = []))
)]
async fn update_profile(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<UpdateProfileRequest>,
) -> Result<Json<UserResponse>> {
    let updated_user = db_users::update_user_profile(
        &app_state.pool,
        &user.id,
        &payload.name,
        &payload.email,
        payload
            .first_day_of_week
            .as_ref()
            .unwrap_or(&user.first_day_of_week),
        payload
            .preferred_units
            .as_ref()
            .unwrap_or(&user.preferred_units),
    )
    .await?;

    Ok(Json(updated_user.into()))
}

async fn update_llm_settings(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<UpdateLlmSettingsRequest>,
) -> Result<Json<UserResponse>> {
    // If api_key is None/empty, preserve existing key (user just didn't re-enter it)
    let api_key = match &payload.api_key {
        Some(k) if !k.is_empty() => Some(k.as_str()),
        Some(_) => user.llm_api_key.as_deref(), // empty string → keep existing
        None => None,                           // explicit null → clear
    };

    let updated_user = db_users::update_user_llm_settings(
        &app_state.pool,
        &user.id,
        payload.base_url.as_deref(),
        api_key,
        payload.model.as_deref(),
    )
    .await?;

    Ok(Json(updated_user.into()))
}

async fn upload_profile_picture(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>)> {

    let mut file_data: Option<Vec<u8>> = None;
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::Validation(validator::ValidationErrors::new()))?
    {
        if field.name().unwrap_or("") == "file" {
            content_type = field.content_type().map(|s| s.to_string());
            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|_| AppError::Validation(validator::ValidationErrors::new()))?
                    .to_vec(),
            );
        }
    }

    let file_data =
        file_data.ok_or_else(|| AppError::Validation(validator::ValidationErrors::new()))?;
    let content_type =
        content_type.ok_or_else(|| AppError::Validation(validator::ValidationErrors::new()))?;

    if !content_type.starts_with("image/") {
        return Err(AppError::Validation(validator::ValidationErrors::new()));
    }

    if file_data.len() > 10 * 1024 * 1024 {
        return Err(AppError::Validation(validator::ValidationErrors::new()));
    }

    let processed = process_uploaded_image(&file_data, &content_type)
        .await
        .map_err(|_| AppError::Validation(validator::ValidationErrors::new()))?;

    db_users::set_user_profile_picture(
        &app_state.pool,
        &user.id,
        &processed.data,
        &processed.content_type,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "success": true,
            "message": "Profile picture updated"
        })),
    ))
}

async fn get_profile_picture(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
) -> Result<Response<Body>> {
    let picture = db_users::get_user_profile_picture(&app_state.pool, &user.id).await?;

    let (data, content_type) = picture.ok_or(AppError::NotFound {
        resource: "Profile picture".to_string(),
    })?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, data.len())
        .header(header::CACHE_CONTROL, "private, max-age=3600")
        .body(Body::from(data))
        .map_err(|_| AppError::Internal {
            message: "Failed to build response".to_string(),
        })?;

    Ok(response)
}

async fn delete_profile_picture(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
) -> Result<StatusCode> {
    db_users::delete_user_profile_picture(&app_state.pool, &user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/auth/change-password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed"),
        (status = 401, description = "Unauthorized or invalid password"),
        (status = 422, description = "Validation error"),
    ),
    security(("session" = []))
)]
async fn change_password(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>> {
    db_users::change_user_password(
        &app_state.pool,
        &user.id,
        &payload.current_password,
        &payload.new_password,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Password updated successfully"
    })))
}

#[utoipa::path(
    get,
    path = "/auth/export",
    responses(
        (status = 200, description = "User data export payload"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("session" = []))
)]
async fn export_data(
    AuthenticatedUser(user): AuthenticatedUser,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let payload = db_users::export_user_data(&app_state.pool, &user.id).await?;
    Ok(Json(payload))
}

#[utoipa::path(
    delete,
    path = "/auth/account",
    request_body = DeleteAccountRequest,
    responses(
        (status = 200, description = "Account deleted"),
        (status = 401, description = "Unauthorized or invalid password"),
        (status = 422, description = "Validation error"),
    ),
    security(("session" = []))
)]
async fn delete_account(
    mut auth_session: AuthSession,
    ValidatedJson(payload): ValidatedJson<DeleteAccountRequest>,
) -> Result<Json<serde_json::Value>> {
    let user = auth_session.user.clone().ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    db_users::delete_user_account(
        &auth_session.backend.db,
        &user.id,
        &payload.current_password,
    )
    .await?;

    let _ = auth_session.logout().await;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Account deleted successfully"
    })))
}
