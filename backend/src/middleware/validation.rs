use axum::{
    async_trait,
    extract::{FromRequest, Request},
    response::Response,
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::utils::errors::{AppError, Result};

#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
    };
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    #[derive(Debug, Deserialize, Serialize, Validate)]
    struct TestRequest {
        #[validate(email)]
        email: String,
        #[validate(length(min = 2))]
        name: String,
        #[validate(range(min = 18, max = 99))]
        age: u32,
    }

    #[derive(Debug, Deserialize, Serialize, Validate)]
    struct InvalidTestRequest {
        #[validate(email)]
        email: String,
        #[validate(length(min = 10))]
        name: String,
    }

    #[tokio::test]
    async fn test_validated_json_valid_request() {
        let test_data = TestRequest {
            email: "test@example.com".to_string(),
            name: "John Doe".to_string(),
            age: 25,
        };

        let json_body = serde_json::to_string(&test_data).unwrap();
        let request = Request::builder()
            .method(http::Method::POST)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(json_body))
            .unwrap();

        let result = ValidatedJson::<TestRequest>::from_request(request, &()).await;
        assert!(result.is_ok());

        let ValidatedJson(extracted_data) = result.unwrap();
        assert_eq!(extracted_data.email, "test@example.com");
        assert_eq!(extracted_data.name, "John Doe");
        assert_eq!(extracted_data.age, 25);
    }

    #[tokio::test]
    async fn test_validated_json_invalid_email() {
        let test_data = TestRequest {
            email: "invalid-email".to_string(),
            name: "John Doe".to_string(),
            age: 25,
        };

        let json_body = serde_json::to_string(&test_data).unwrap();
        let request = Request::builder()
            .method(http::Method::POST)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(json_body))
            .unwrap();

        let result = ValidatedJson::<TestRequest>::from_request(request, &()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::Validation(validation_errors) => {
                assert!(validation_errors.field_errors().contains_key("email"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[tokio::test]
    async fn test_validated_json_invalid_name_length() {
        let test_data = TestRequest {
            email: "test@example.com".to_string(),
            name: "A".to_string(), // Too short
            age: 25,
        };

        let json_body = serde_json::to_string(&test_data).unwrap();
        let request = Request::builder()
            .method(http::Method::POST)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(json_body))
            .unwrap();

        let result = ValidatedJson::<TestRequest>::from_request(request, &()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::Validation(validation_errors) => {
                assert!(validation_errors.field_errors().contains_key("name"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[tokio::test]
    async fn test_validated_json_invalid_age_range() {
        let test_data = TestRequest {
            email: "test@example.com".to_string(),
            name: "John Doe".to_string(),
            age: 17, // Below minimum age
        };

        let json_body = serde_json::to_string(&test_data).unwrap();
        let request = Request::builder()
            .method(http::Method::POST)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(json_body))
            .unwrap();

        let result = ValidatedJson::<TestRequest>::from_request(request, &()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::Validation(validation_errors) => {
                assert!(validation_errors.field_errors().contains_key("age"));
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[tokio::test]
    async fn test_validated_json_multiple_validation_errors() {
        let test_data = InvalidTestRequest {
            email: "invalid-email".to_string(),
            name: "short".to_string(), // Too short (needs min 10 chars)
        };

        let json_body = serde_json::to_string(&test_data).unwrap();
        let request = Request::builder()
            .method(http::Method::POST)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(json_body))
            .unwrap();

        let result = ValidatedJson::<InvalidTestRequest>::from_request(request, &()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::Validation(validation_errors) => {
                let field_errors = validation_errors.field_errors();
                assert!(field_errors.contains_key("email"));
                assert!(field_errors.contains_key("name"));
                assert_eq!(field_errors.len(), 2);
            }
            _ => panic!("Expected validation error"),
        }
    }

    #[tokio::test]
    async fn test_validated_json_invalid_json() {
        let invalid_json = "{ invalid json }";
        let request = Request::builder()
            .method(http::Method::POST)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(invalid_json))
            .unwrap();

        let result = ValidatedJson::<TestRequest>::from_request(request, &()).await;
        assert!(result.is_err());

        // Should get JsonRejection error from the JSON extraction before validation
        let error = result.unwrap_err();
        match error {
            AppError::JsonRejection(_) => {
                // This is expected - JSON parsing failed before validation
            }
            _ => panic!("Expected JsonRejection error, got: {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_validated_json_missing_content_type() {
        let test_data = TestRequest {
            email: "test@example.com".to_string(),
            name: "John Doe".to_string(),
            age: 25,
        };

        let json_body = serde_json::to_string(&test_data).unwrap();
        let request = Request::builder()
            .method(http::Method::POST)
            // Missing Content-Type header
            .body(Body::from(json_body))
            .unwrap();

        let result = ValidatedJson::<TestRequest>::from_request(request, &()).await;
        assert!(result.is_err());

        // Should get JsonRejection error due to missing content type
        let error = result.unwrap_err();
        match error {
            AppError::JsonRejection(_) => {
                // This is expected
            }
            _ => panic!("Expected JsonRejection error, got: {:?}", error),
        }
    }

    #[test]
    fn test_validated_json_debug() {
        let test_data = TestRequest {
            email: "test@example.com".to_string(),
            name: "John Doe".to_string(),
            age: 25,
        };

        let validated_json = ValidatedJson(test_data);
        let debug_output = format!("{:?}", validated_json);
        assert!(debug_output.contains("ValidatedJson"));
        assert!(debug_output.contains("test@example.com"));
        assert!(debug_output.contains("John Doe"));
    }
}