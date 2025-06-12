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
