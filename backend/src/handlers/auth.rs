use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};

use crate::models::{AuthResponse, CreateUserRequest, LoginRequest, UserResponse};

pub fn routes() -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/me", get(me))
}

async fn login(Json(payload): Json<LoginRequest>) -> Result<Json<AuthResponse>, StatusCode> {
    // TODO: Implement actual authentication
    tracing::info!("Login attempt for email: {}", payload.email);
    
    // Mock response for now
    let mock_user = UserResponse {
        id: uuid::Uuid::new_v4(),
        email: payload.email,
        name: "Mock User".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let response = AuthResponse {
        token: "mock-jwt-token".to_string(),
        user: mock_user,
    };
    
    Ok(Json(response))
}

async fn register(Json(payload): Json<CreateUserRequest>) -> Result<Json<AuthResponse>, StatusCode> {
    // TODO: Implement actual user registration
    tracing::info!("Registration attempt for email: {}", payload.email);
    
    // Mock response for now
    let mock_user = UserResponse {
        id: uuid::Uuid::new_v4(),
        email: payload.email,
        name: payload.name,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let response = AuthResponse {
        token: "mock-jwt-token".to_string(),
        user: mock_user,
    };
    
    Ok(Json(response))
}

async fn me() -> Result<Json<UserResponse>, StatusCode> {
    // TODO: Implement actual user profile retrieval
    tracing::info!("Profile request");
    
    // Mock response for now
    let mock_user = UserResponse {
        id: uuid::Uuid::new_v4(),
        email: "user@example.com".to_string(),
        name: "Mock User".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    Ok(Json(mock_user))
}