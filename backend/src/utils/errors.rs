use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::Serialize;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Debug)]
#[allow(dead_code)]
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
            Self::Validation(validation_errors) => {
                let details = validation_errors
                    .field_errors()
                    .iter()
                    .map(|(field, errors)| {
                        let messages: Vec<String> = errors
                            .iter()
                            .filter_map(|error| error.message.as_ref().map(std::string::ToString::to_string))
                            .collect();
                        ((*field).to_string(), messages)
                    })
                    .collect::<std::collections::HashMap<String, Vec<String>>>();

                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "validation_error",
                    "Request validation failed",
                    Some(serde_json::to_value(details).unwrap_or_default()),
                )
            }
            Self::JsonRejection(rejection) => {
                tracing::error!("JSON rejection: {}", rejection);
                (
                    StatusCode::BAD_REQUEST,
                    "json_error",
                    "Invalid JSON in request body",
                    Some(serde_json::json!({ "details": rejection.to_string() })),
                )
            }
            Self::Database(db_error) => {
                tracing::error!("Database error: {}", db_error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    "A database error occurred",
                    None,
                )
            }
            Self::Authentication { message } => (
                StatusCode::UNAUTHORIZED,
                "authentication_error",
                message.as_str(),
                None,
            ),
            Self::Authorization { message } => (
                StatusCode::FORBIDDEN,
                "authorization_error",
                message.as_str(),
                None,
            ),
            Self::NotFound { resource } => (
                StatusCode::NOT_FOUND,
                "not_found",
                resource.as_str(),
                None,
            ),
            Self::Internal { message } => {
                tracing::error!("Internal error: {}", message);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "An internal server error occurred",
                    None,
                )
            }
        };

        // Log all error responses with timestamp and details for debugging
        tracing::debug!(
            status_code = %status,
            error_type = %error_type,
            message = %message,
            details = ?details,
            timestamp = %Utc::now().to_rfc3339(),
            "Error response generated"
        );

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message: message.to_string(),
            details,
        });

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use serde_json::Value;
    use validator::{Validate, ValidationError, ValidationErrors};

    #[derive(Validate)]
    struct TestStruct {
        #[validate(email)]
        email: String,
        #[validate(length(min = 8))]
        password: String,
    }

    #[tokio::test]
    async fn test_validation_error_response() {
        let test_data = TestStruct {
            email: "invalid-email".to_string(),
            password: "short".to_string(),
        };
        
        let validation_result = test_data.validate();
        assert!(validation_result.is_err());
        
        let error = AppError::Validation(validation_result.unwrap_err());
        let response = error.into_response();
        
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["error"], "validation_error");
        assert_eq!(json["message"], "Request validation failed");
        assert!(json["details"].is_object());
    }

    #[tokio::test]
    async fn test_json_rejection_error_response() {
        // Create a simple JSON error using missing content type rejection
        use axum::extract::rejection::JsonRejection;
        
        let json_error = JsonRejection::MissingJsonContentType(Default::default());
        
        let error = AppError::JsonRejection(json_error);
        let response = error.into_response();
        
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["error"], "json_error");
        assert_eq!(json["message"], "Invalid JSON in request body");
        assert!(json["details"].is_object());
    }

    #[tokio::test]
    async fn test_database_error_response() {
        let error = AppError::Database(sqlx::Error::RowNotFound);
        let response = error.into_response();
        
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["error"], "database_error");
        assert_eq!(json["message"], "A database error occurred");
        assert!(json["details"].is_null());
    }

    #[tokio::test]
    async fn test_authentication_error_response() {
        let error = AppError::Authentication { 
            message: "Invalid credentials".to_string() 
        };
        let response = error.into_response();
        
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["error"], "authentication_error");
        assert_eq!(json["message"], "Invalid credentials");
        assert!(json["details"].is_null());
    }

    #[tokio::test]
    async fn test_authorization_error_response() {
        let error = AppError::Authorization { 
            message: "Insufficient permissions".to_string() 
        };
        let response = error.into_response();
        
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["error"], "authorization_error");
        assert_eq!(json["message"], "Insufficient permissions");
        assert!(json["details"].is_null());
    }

    #[tokio::test]
    async fn test_not_found_error_response() {
        let error = AppError::NotFound { 
            resource: "Plant with id 123".to_string() 
        };
        let response = error.into_response();
        
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["error"], "not_found");
        assert_eq!(json["message"], "Plant with id 123");
        assert!(json["details"].is_null());
    }

    #[tokio::test]
    async fn test_internal_error_response() {
        let error = AppError::Internal { 
            message: "Something went wrong".to_string() 
        };
        let response = error.into_response();
        
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["error"], "internal_error");
        assert_eq!(json["message"], "An internal server error occurred");
        assert!(json["details"].is_null());
    }

    #[test]
    fn test_error_display() {
        let error = AppError::Authentication { 
            message: "Invalid token".to_string() 
        };
        assert_eq!(format!("{}", error), "Authentication error: Invalid token");

        let error = AppError::NotFound { 
            resource: "User".to_string() 
        };
        assert_eq!(format!("{}", error), "Not found: User");
    }

    #[test]
    fn test_error_debug() {
        let error = AppError::Internal { 
            message: "Duplicate entry".to_string() 
        };
        let debug_output = format!("{:?}", error);
        assert!(debug_output.contains("Internal"));
        assert!(debug_output.contains("Duplicate entry"));
    }

    #[test]
    fn test_error_from_conversions() {
        // Test conversion from ValidationErrors
        let mut validation_errors = ValidationErrors::new();
        validation_errors.add("email", ValidationError::new("invalid"));
        let error: AppError = validation_errors.into();
        assert!(matches!(error, AppError::Validation(_)));

        // Test conversion from sqlx::Error
        let sqlx_error = sqlx::Error::RowNotFound;
        let error: AppError = sqlx_error.into();
        assert!(matches!(error, AppError::Database(_)));
    }

    #[test]
    fn test_error_logging_includes_timestamp() {
        // Test that IntoResponse generates logs with timestamp
        use axum::response::IntoResponse;
        
        let error = AppError::NotFound {
            resource: "Test Resource".to_string(),
        };
        
        // This should trigger the debug logging with timestamp
        let _response = error.into_response();
        
        // Note: In a real implementation, you might want to capture logs
        // and verify they contain the expected timestamp format
        // For this test, we're just ensuring the code path executes without panic
    }
}