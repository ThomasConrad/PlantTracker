use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use validator::Validate;

// For axum-login integration
use axum_login::AuthUser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String, // Changed to String for SQLite compatibility
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub salt: String,
    pub role: UserRole,
    pub can_create_invites: bool,
    pub max_invites: Option<i32>, // None means unlimited
    pub invites_created: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Moderator,
    User,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::Moderator => write!(f, "moderator"),
            UserRole::User => write!(f, "user"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(UserRole::Admin),
            "moderator" => Ok(UserRole::Moderator),
            "user" => Ok(UserRole::User),
            _ => Err(format!("Invalid user role: {}", s)),
        }
    }
}

#[derive(Debug, FromRow)]
pub struct UserRow {
    pub id: String,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub salt: String,
    pub role: String,
    pub can_create_invites: bool,
    pub max_invites: Option<i32>,
    pub invites_created: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl UserRow {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_user(self) -> Result<User, crate::utils::errors::AppError> {
        Ok(User {
            id: self.id,
            email: self.email,
            name: self.name,
            password_hash: self.password_hash,
            salt: self.salt,
            role: self.role.parse().map_err(|e| {
                crate::utils::errors::AppError::Internal {
                    message: format!("Invalid user role in database: {}", e),
                }
            })?,
            can_create_invites: self.can_create_invites,
            max_invites: self.max_invites,
            invites_created: self.invites_created,
            created_at: self.created_at.parse::<DateTime<Utc>>().map_err(|_| {
                crate::utils::errors::AppError::Internal {
                    message: "Invalid datetime in database".to_string(),
                }
            })?,
            updated_at: self.updated_at.parse::<DateTime<Utc>>().map_err(|_| {
                crate::utils::errors::AppError::Internal {
                    message: "Invalid datetime in database".to_string(),
                }
            })?,
        })
    }
}

// Implement AuthUser for axum-login integration
impl AuthUser for User {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 2))]
    pub name: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub invite_code: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: UserRole,
    pub can_create_invites: bool,
    pub max_invites: Option<i32>,
    pub invites_created: i32,
    pub invites_remaining: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub user: UserResponse,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin
    }

    #[allow(dead_code)]
    pub fn is_moderator_or_above(&self) -> bool {
        matches!(self.role, UserRole::Admin | UserRole::Moderator)
    }

    #[allow(dead_code)]
    pub fn can_create_invite(&self) -> bool {
        self.can_create_invites && self.has_invites_remaining()
    }

    #[allow(dead_code)]
    pub fn has_invites_remaining(&self) -> bool {
        match self.max_invites {
            None => true, // Unlimited invites
            Some(max) => self.invites_created < max,
        }
    }

    #[allow(dead_code)]
    pub fn invites_remaining(&self) -> Option<i32> {
        self.max_invites.map(|max| max - self.invites_created)
    }
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        let invites_remaining = user.max_invites.map(|max| max - user.invites_created);
        
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role,
            can_create_invites: user.can_create_invites,
            max_invites: user.max_invites,
            invites_created: user.invites_created,
            invites_remaining,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_create_user_request_validation_valid() {
        let request = CreateUserRequest {
            email: "test@example.com".to_string(),
            name: "John Doe".to_string(),
            password: "password123".to_string(),
            invite_code: Some("INVITE123".to_string()),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_user_request_validation_invalid_email() {
        let request = CreateUserRequest {
            email: "invalid-email".to_string(),
            name: "John Doe".to_string(),
            password: "password123".to_string(),
            invite_code: Some("INVITE123".to_string()),
        };

        let validation_result = request.validate();
        assert!(validation_result.is_err());

        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("email"));
    }

    #[test]
    fn test_create_user_request_validation_short_name() {
        let request = CreateUserRequest {
            email: "test@example.com".to_string(),
            name: "A".to_string(), // Too short (minimum 2 characters)
            password: "password123".to_string(),
            invite_code: Some("INVITE123".to_string()),
        };

        let validation_result = request.validate();
        assert!(validation_result.is_err());

        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("name"));
    }

    #[test]
    fn test_create_user_request_validation_short_password() {
        let request = CreateUserRequest {
            email: "test@example.com".to_string(),
            name: "John Doe".to_string(),
            password: "short".to_string(), // Too short (minimum 8 characters)
            invite_code: Some("INVITE123".to_string()),
        };

        let validation_result = request.validate();
        assert!(validation_result.is_err());

        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("password"));
    }

    #[test]
    fn test_login_request_validation_valid() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_login_request_validation_invalid_email() {
        let request = LoginRequest {
            email: "not-an-email".to_string(),
            password: "password123".to_string(),
        };

        let validation_result = request.validate();
        assert!(validation_result.is_err());

        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("email"));
    }

    #[test]
    fn test_user_auth_user_implementation() {
        let user = User {
            id: "test-id".to_string(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            password_hash: "hashed_password".to_string(),
            salt: "salt".to_string(),
            role: UserRole::User,
            can_create_invites: false,
            max_invites: Some(5),
            invites_created: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Test AuthUser trait implementation
        assert_eq!(user.id(), "test-id");
        assert_eq!(user.session_auth_hash(), "hashed_password".as_bytes());
    }

    #[test]
    fn test_user_response_from_user() {
        let user = User {
            id: "test-id".to_string(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            password_hash: "hashed_password".to_string(),
            salt: "salt".to_string(),
            role: UserRole::User,
            can_create_invites: false,
            max_invites: Some(5),
            invites_created: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response = UserResponse::from(user.clone());

        assert_eq!(response.id, user.id);
        assert_eq!(response.email, user.email);
        assert_eq!(response.name, user.name);
        assert_eq!(response.created_at, user.created_at);
        assert_eq!(response.updated_at, user.updated_at);
    }

    #[test]
    fn test_auth_response_serialization() {
        let user_response = UserResponse {
            id: "test-id".to_string(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            role: UserRole::User,
            can_create_invites: false,
            max_invites: Some(5),
            invites_created: 0,
            invites_remaining: Some(5),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let auth_response = AuthResponse {
            user: user_response,
        };

        let json = serde_json::to_string(&auth_response).unwrap();
        assert!(json.contains("\"user\":{"));
        assert!(json.contains("\"id\":\"test-id\""));
        assert!(json.contains("\"email\":\"test@example.com\""));
        assert!(json.contains("\"name\":\"Test User\""));
    }

    #[test]
    fn test_user_row_to_user_valid() {
        let user_row = UserRow {
            id: "test-id".to_string(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            password_hash: "hashed_password".to_string(),
            salt: "salt".to_string(),
            role: "user".to_string(),
            can_create_invites: false,
            max_invites: Some(5),
            invites_created: 0,
            created_at: "2024-01-01T12:00:00Z".to_string(),
            updated_at: "2024-01-01T12:00:00Z".to_string(),
        };

        let user = user_row.to_user().unwrap();

        assert_eq!(user.id, "test-id");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.name, "Test User");
        assert_eq!(user.password_hash, "hashed_password");
        assert_eq!(user.salt, "salt");
    }

    #[test]
    fn test_user_row_to_user_invalid_datetime() {
        let user_row = UserRow {
            id: "test-id".to_string(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            password_hash: "hashed_password".to_string(),
            salt: "salt".to_string(),
            role: "user".to_string(),
            can_create_invites: false,
            max_invites: Some(5),
            invites_created: 0,
            created_at: "invalid-datetime".to_string(),
            updated_at: "2024-01-01T12:00:00Z".to_string(),
        };

        let result = user_row.to_user();
        assert!(result.is_err());

        if let Err(crate::utils::errors::AppError::Internal { message }) = result {
            assert_eq!(message, "Invalid datetime in database");
        } else {
            panic!("Expected Internal error with datetime message");
        }
    }

    #[test]
    fn test_user_clone() {
        let user = User {
            id: "test-id".to_string(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            password_hash: "hashed_password".to_string(),
            salt: "salt".to_string(),
            role: UserRole::User,
            can_create_invites: false,
            max_invites: Some(5),
            invites_created: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let cloned_user = user.clone();
        assert_eq!(user.id, cloned_user.id);
        assert_eq!(user.email, cloned_user.email);
        assert_eq!(user.name, cloned_user.name);
        assert_eq!(user.password_hash, cloned_user.password_hash);
        assert_eq!(user.salt, cloned_user.salt);
    }

    #[test]
    fn test_user_debug_format() {
        let user = User {
            id: "test-id".to_string(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            password_hash: "hashed_password".to_string(),
            salt: "salt".to_string(),
            role: UserRole::User,
            can_create_invites: false,
            max_invites: Some(5),
            invites_created: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let debug_output = format!("{:?}", user);
        assert!(debug_output.contains("User"));
        assert!(debug_output.contains("test@example.com"));
        assert!(debug_output.contains("Test User"));
        // Password hash should be included in debug but we don't want to assert on the exact value
        assert!(debug_output.contains("password_hash"));
    }
}
