use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationErrors),
    #[error("JSON parsing error: {0}")]
    JsonRejection(#[from] axum::extract::rejection::JsonRejection),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Authentication error: {message}")]
    Authentication { message: String },
    #[error("Authorization error: {message}")]
    Authorization { message: String },
    #[error("Not found: {resource}")]
    NotFound { resource: String },
    #[error("Internal server error: {message}")]
    Internal { message: String },
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message, details) = match &self {
            AppError::Validation(validation_errors) => {
                let details = validation_errors
                    .field_errors()
                    .iter()
                    .map(|(field, errors)| {
                        let messages: Vec<String> = errors
                            .iter()
                            .filter_map(|error| error.message.as_ref().map(|m| m.to_string()))
                            .collect();
                        (field.to_string(), messages)
                    })
                    .collect::<std::collections::HashMap<String, Vec<String>>>();

                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "validation_error",
                    "Request validation failed",
                    Some(serde_json::to_value(details).unwrap_or_default()),
                )
            }
            AppError::JsonRejection(rejection) => {
                tracing::error!("JSON rejection: {}", rejection);
                (
                    StatusCode::BAD_REQUEST,
                    "json_error",
                    "Invalid JSON in request body",
                    Some(serde_json::json!({ "details": rejection.to_string() })),
                )
            }
            AppError::Database(db_error) => {
                tracing::error!("Database error: {}", db_error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    "A database error occurred",
                    None,
                )
            }
            AppError::Authentication { message } => (
                StatusCode::UNAUTHORIZED,
                "authentication_error",
                message.as_str(),
                None,
            ),
            AppError::Authorization { message } => (
                StatusCode::FORBIDDEN,
                "authorization_error",
                message.as_str(),
                None,
            ),
            AppError::NotFound { resource } => (
                StatusCode::NOT_FOUND,
                "not_found",
                resource.as_str(),
                None,
            ),
            AppError::Internal { message } => {
                tracing::error!("Internal error: {}", message);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "An internal server error occurred",
                    None,
                )
            }
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message: message.to_string(),
            details,
        });

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;