use axum::{
    response::Json,
    routing::{delete, get, post, put},
    Router,
};

use crate::app_state::AppState;
use crate::auth::{AuthSession, Credentials};
use crate::database::users as db_users;
use crate::middleware::validation::ValidatedJson;
use crate::models::{
    AuthResponse, ChangePasswordRequest, CreateUserRequest, DeleteAccountRequest, LoginRequest,
    UpdateProfileRequest, UserResponse, UserRole,
};
use crate::utils::errors::{AppError, Result};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/profile", put(update_profile))
        .route("/change-password", post(change_password))
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
        // No invite code provided - registration not allowed
        return Err(AppError::Authentication {
            message: "Registration requires a valid invite code".to_string(),
        });
    };

    // Create user in database with appropriate role
    let user = if is_admin_invite {
        // Create admin user with unlimited invites
        db_users::create_user_internal(
            &auth_session.backend.db,
            &payload,
            UserRole::Admin,
            true,
            None, // Unlimited invites for admin
        )
        .await
    } else {
        // Create regular user
        db_users::create_user(&auth_session.backend.db, &payload).await
    }?;

    // Mark invite code as used
    use crate::database::invites as db_invites;

    if let Err(e) =
        db_invites::use_invite_code(&auth_session.backend.db, &invite_code, &user.id).await
    {
        tracing::error!("Failed to mark invite code as used: {}", e);
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

async fn me(auth_session: AuthSession) -> Result<Json<UserResponse>> {
    if let Some(user) = auth_session.user {
        tracing::debug!("Retrieved user profile: {}", user.email);
        Ok(Json(user.into()))
    } else {
        tracing::warn!("Unauthenticated request to /me endpoint");
        Err(AppError::Authentication {
            message: "Not authenticated".to_string(),
        })
    }
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
    auth_session: AuthSession,
    ValidatedJson(payload): ValidatedJson<UpdateProfileRequest>,
) -> Result<Json<UserResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let updated_user = db_users::update_user_profile(
        &auth_session.backend.db,
        &user.id,
        &payload.name,
        &payload.email,
    )
    .await?;

    Ok(Json(updated_user.into()))
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
    auth_session: AuthSession,
    ValidatedJson(payload): ValidatedJson<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    db_users::change_user_password(
        &auth_session.backend.db,
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
async fn export_data(auth_session: AuthSession) -> Result<Json<serde_json::Value>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Not authenticated".to_string(),
    })?;

    let payload = db_users::export_user_data(&auth_session.backend.db, &user.id).await?;
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
