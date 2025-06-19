use axum::{
    response::Json,
    routing::{get, post},
    Router,
};

use crate::app_state::AppState;
use crate::auth::{AuthSession, Credentials};
use crate::database::users as db_users;
use crate::middleware::validation::ValidatedJson;
use crate::models::{AuthResponse, CreateUserRequest, LoginRequest, UserResponse};
use crate::utils::errors::{AppError, Result};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/logout", post(logout))
        .route("/me", get(me))
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
    if let Some(invite_code) = &payload.invite_code {
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
    } else {
        // No invite code provided - registration not allowed
        return Err(AppError::Authentication {
            message: "Registration requires a valid invite code".to_string(),
        });
    }

    // Create user in database
    let user = db_users::create_user(&auth_session.backend.db, &payload)
        .await
        .map_err(|e| {
            match e {
                AppError::Validation(_) => AppError::Validation(
                    // TODO: Create proper validation error for email already exists
                    validator::ValidationErrors::new(),
                ),
                _ => e,
            }
        })?;

    // Mark invite code as used
    if let Some(invite_code) = &payload.invite_code {
        use crate::database::invites as db_invites;
        
        if let Err(e) = db_invites::use_invite_code(&auth_session.backend.db, invite_code, &user.id).await {
            tracing::error!("Failed to mark invite code as used: {}", e);
            // Don't fail registration if we can't update invite code
        }

        // Update waitlist status if user was on waitlist
        if let Err(e) = db_invites::update_waitlist_status(
            &auth_session.backend.db, 
            &payload.email, 
            "registered", 
            Some(invite_code)
        ).await {
            tracing::debug!("User was not on waitlist or failed to update status: {}", e);
            // This is fine - user might not have been on waitlist
        }
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
