//! Axum extractors for authentication.
//!
//! `AuthenticatedUser` replaces the repeated pattern of:
//! ```ignore
//! let user = auth_session.user.ok_or(AppError::Authentication {
//!     message: "Not authenticated".to_string(),
//! })?;
//! ```
//!
//! Usage in handlers:
//! ```ignore
//! async fn my_handler(AuthenticatedUser(user): AuthenticatedUser) -> Result<...> {
//!     // `user` is guaranteed to be a valid User
//! }
//! ```

use axum::{extract::FromRequestParts, http::request::Parts};

use crate::auth::AuthSession;
use crate::models::User;
use crate::utils::errors::AppError;

/// Axum extractor that guarantees an authenticated user.
///
/// Rejects with `401 Unauthorized` if no user session exists.
/// Destructure in handler signature: `AuthenticatedUser(user): AuthenticatedUser`
pub struct AuthenticatedUser(pub User);

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_session = AuthSession::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Authentication {
                message: "Not authenticated".to_string(),
            })?;

        let user = auth_session.user.ok_or(AppError::Authentication {
            message: "Not authenticated".to_string(),
        })?;

        Ok(Self(user))
    }
}
