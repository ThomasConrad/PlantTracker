use reqwest::StatusCode;
use serde_json::{json, Value};

mod common;

use common::{TestApp, create_test_user, login_user};

#[tokio::test]
async fn test_google_tasks_auth_url_requires_authentication() {
    let app = TestApp::new().await;
    
    let response = app
        .client
        .get(&format!("{}/google-tasks/auth-url", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_google_tasks_status_not_connected() {
    let app = TestApp::new().await;
    let _user = create_test_user(&app, "test@example.com", "Test User", "password123").await;
    login_user(&app, "test@example.com", "password123").await;
    
    let response = app
        .client
        .get(&format!("{}/google-tasks/status", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse response");
    
    assert_eq!(body["connected"], false);
    assert!(body["connected_at"].is_null());
    assert!(body["scopes"].is_null());
    assert!(body["expires_at"].is_null());
}

#[tokio::test]
async fn test_google_tasks_store_tokens() {
    let app = TestApp::new().await;
    let _user = create_test_user(&app, "test@example.com", "Test User", "password123").await;
    login_user(&app, "test@example.com", "password123").await;
    
    let request_body = json!({
        "access_token": "test_access_token",
        "refresh_token": "test_refresh_token",
        "expires_at": 1234567890
    });
    
    let response = app
        .client
        .post(&format!("{}/google-tasks/store-tokens", app.address))
        .json(&request_body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse response");
    
    assert_eq!(body["success"], true);
    assert_eq!(body["message"], "Google Tasks integration configured successfully");
    assert!(body["connected_at"].is_string());
    assert_eq!(body["scopes"][0], "https://www.googleapis.com/auth/tasks");
}

#[tokio::test]
async fn test_google_tasks_disconnect_not_connected() {
    let app = TestApp::new().await;
    let _user = create_test_user(&app, "test@example.com", "Test User", "password123").await;
    login_user(&app, "test@example.com", "password123").await;
    
    let response = app
        .client
        .post(&format!("{}/google-tasks/disconnect", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["error"], "not_found");
    assert!(body["message"].as_str().unwrap().contains("Google Tasks connection"));
}

#[tokio::test]
async fn test_google_tasks_sync_requires_connection() {
    let app = TestApp::new().await;
    let _user = create_test_user(&app, "test@example.com", "Test User", "password123").await;
    login_user(&app, "test@example.com", "password123").await;
    
    let sync_request = json!({
        "days_ahead": 30
    });
    
    let response = app
        .client
        .post(&format!("{}/google-tasks/sync-tasks", app.address))
        .json(&sync_request)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    let body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["error"], "authentication_error");
    assert!(body["message"].as_str().unwrap().contains("Google Tasks connection"));
}

#[tokio::test]
async fn test_google_tasks_create_task_requires_connection() {
    let app = TestApp::new().await;
    let _user = create_test_user(&app, "test@example.com", "Test User", "password123").await;
    login_user(&app, "test@example.com", "password123").await;
    
    let task_request = json!({
        "title": "💧 Water Fiddle Leaf Fig",
        "notes": "Time to water your plant",
        "due_time": "2024-01-15T10:00:00Z"
    });
    
    let response = app
        .client
        .post(&format!("{}/google-tasks/create-task", app.address))
        .json(&task_request)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    let body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["error"], "authentication_error");
    assert!(body["message"].as_str().unwrap().contains("Google Tasks connection"));
}

#[tokio::test]
async fn test_google_tasks_error_handling() {
    let app = TestApp::new().await;
    let _user = create_test_user(&app, "test@example.com", "Test User", "password123").await;
    login_user(&app, "test@example.com", "password123").await;
    
    // Test invalid JSON in store tokens request
    let response = app
        .client
        .post(&format!("{}/google-tasks/store-tokens", app.address))
        .header("content-type", "application/json")
        .body("invalid json")
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["error"], "json_error");
}

#[tokio::test]
async fn test_google_tasks_json_data_format() {
    // Test basic JSON formatting for tasks
    use chrono::Utc;
    
    let due_time = Utc::now();
    let task_data = json!({
        "title": "💧 Water Test Plant",
        "notes": "Time to water your plant",
        "due": due_time.to_rfc3339(),
        "status": "needsAction"
    });
    
    assert_eq!(task_data["title"], "💧 Water Test Plant");
    assert_eq!(task_data["notes"], "Time to water your plant");
    assert_eq!(task_data["status"], "needsAction");
    assert!(task_data["due"].is_string());
}

#[tokio::test]
async fn test_google_tasks_database_integration() {
    let app = TestApp::new().await;
    let user_response = create_test_user(&app, "test@example.com", "Test User", "password123").await;
    login_user(&app, "test@example.com", "password123").await;
    let user_id = user_response["user"]["id"].as_str().unwrap();
    
    // Test storing and retrieving OAuth tokens directly from database
    use plant_tracker_api::database::google_oauth;
    use chrono::Utc;
    
    let scope = "https://www.googleapis.com/auth/tasks";
    let expires_at = Some(Utc::now() + chrono::Duration::hours(1));
    
    // Store token
    let result = google_oauth::save_oauth_token(
        &app.db_pool,
        user_id,
        "test_access_token",
        Some("test_refresh_token"),
        expires_at,
        scope,
    ).await;
    
    assert!(result.is_ok());
    let stored_token = result.unwrap();
    assert_eq!(stored_token.access_token, "test_access_token");
    assert_eq!(stored_token.refresh_token, Some("test_refresh_token".to_string()));
    assert_eq!(stored_token.scope, scope);
    
    // Retrieve token
    let retrieved_token = google_oauth::get_oauth_token(&app.db_pool, user_id).await;
    assert!(retrieved_token.is_ok());
    
    let token = retrieved_token.unwrap();
    assert!(token.is_some());
    
    let token = token.unwrap();
    assert_eq!(token.access_token, "test_access_token");
    assert_eq!(token.refresh_token, Some("test_refresh_token".to_string()));
    assert_eq!(token.scope, scope);
    
    // Test token validation
    let is_valid = google_oauth::has_valid_token(&app.db_pool, user_id).await;
    assert!(is_valid.is_ok());
    assert_eq!(is_valid.unwrap(), true);
    
    // Delete token
    let delete_result = google_oauth::delete_oauth_token(&app.db_pool, user_id).await;
    assert!(delete_result.is_ok());
    
    // Verify token is gone
    let deleted_token = google_oauth::get_oauth_token(&app.db_pool, user_id).await;
    assert!(deleted_token.is_ok());
    assert!(deleted_token.unwrap().is_none());
}