use serde_json::json;

mod common;
use common::TestApp;

#[tokio::test]
async fn test_user_registration() {
    let app = TestApp::new().await;

    let response = app
        .client
        .post(&app.url("/auth/register"))
        .json(&json!({
            "email": "test@example.com",
            "name": "Test User",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 201);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["user"]["email"], "test@example.com");
    assert_eq!(body["user"]["name"], "Test User");
    assert!(body["user"]["id"].is_string());
}

#[tokio::test]
async fn test_user_registration_duplicate_email() {
    let app = TestApp::new().await;

    // First registration should succeed
    let response1 = app
        .client
        .post(&app.url("/auth/register"))
        .json(&json!({
            "email": "duplicate@example.com",
            "name": "First User",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send first request");

    assert_eq!(response1.status(), 201);

    // Second registration with same email should fail
    let response2 = app
        .client
        .post(&app.url("/auth/register"))
        .json(&json!({
            "email": "duplicate@example.com",
            "name": "Second User",
            "password": "different_password"
        }))
        .send()
        .await
        .expect("Failed to send second request");

    assert_eq!(response2.status(), 422); // Validation error
}

#[tokio::test]
async fn test_user_login_success() {
    let app = TestApp::new().await;

    // Register a user first
    common::create_test_user(&app, "login@example.com", "Login User", "password123").await;

    // Now try to login
    let response = app
        .client
        .post(&app.url("/auth/login"))
        .json(&json!({
            "email": "login@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["user"]["email"], "login@example.com");
    assert_eq!(body["user"]["name"], "Login User");
}

#[tokio::test]
async fn test_user_login_invalid_credentials() {
    let app = TestApp::new().await;

    // Register a user first
    common::create_test_user(&app, "valid@example.com", "Valid User", "correct_password").await;

    // Try to login with wrong password
    let response = app
        .client
        .post(&app.url("/auth/login"))
        .json(&json!({
            "email": "valid@example.com",
            "password": "wrong_password"
        }))
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(response.status(), 401); // Unauthorized
}

#[tokio::test]
async fn test_user_login_nonexistent_user() {
    let app = TestApp::new().await;

    let response = app
        .client
        .post(&app.url("/auth/login"))
        .json(&json!({
            "email": "nonexistent@example.com",
            "password": "any_password"
        }))
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(response.status(), 401); // Unauthorized
}

#[tokio::test]
async fn test_me_endpoint_authenticated() {
    let app = TestApp::new().await;

    // Register and login
    common::create_test_user(&app, "me@example.com", "Me User", "password123").await;

    // Get user profile
    let response = app
        .client
        .get(&app.url("/auth/me"))
        .send()
        .await
        .expect("Failed to send me request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["email"], "me@example.com");
    assert_eq!(body["name"], "Me User");
}

#[tokio::test]
async fn test_me_endpoint_unauthenticated() {
    let app = TestApp::new().await;

    // Try to access /me without authentication
    let response = app
        .client
        .get(&app.url("/auth/me"))
        .send()
        .await
        .expect("Failed to send me request");

    assert_eq!(response.status(), 401); // Unauthorized
}

#[tokio::test]
async fn test_logout() {
    let app = TestApp::new().await;

    // Register and login
    common::create_test_user(&app, "logout@example.com", "Logout User", "password123").await;

    // Verify we're authenticated
    let me_response = app
        .client
        .get(&app.url("/auth/me"))
        .send()
        .await
        .expect("Failed to send me request");
    assert_eq!(me_response.status(), 200);

    // Logout
    let logout_response = app
        .client
        .post(&app.url("/auth/logout"))
        .send()
        .await
        .expect("Failed to send logout request");
    assert_eq!(logout_response.status(), 200);

    // Verify we're no longer authenticated
    let me_response_after = app
        .client
        .get(&app.url("/auth/me"))
        .send()
        .await
        .expect("Failed to send me request after logout");
    assert_eq!(me_response_after.status(), 401);
}

#[tokio::test]
async fn test_session_persistence() {
    let app = TestApp::new().await;

    // Register and login
    common::create_test_user(&app, "session@example.com", "Session User", "password123").await;

    // Make multiple requests to verify session persists
    for _ in 0..3 {
        let response = app
            .client
            .get(&app.url("/auth/me"))
            .send()
            .await
            .expect("Failed to send me request");

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.expect("Failed to parse response");
        assert_eq!(body["email"], "session@example.com");
    }
}

#[tokio::test]
async fn test_validation_errors() {
    let app = TestApp::new().await;

    // Test invalid email format
    let response = app
        .client
        .post(&app.url("/auth/register"))
        .json(&json!({
            "email": "not-an-email",
            "name": "Test User",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 422); // Validation error

    // Test short password
    let response = app
        .client
        .post(&app.url("/auth/register"))
        .json(&json!({
            "email": "valid@example.com",
            "name": "Test User",
            "password": "short"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 422); // Validation error

    // Test empty name
    let response = app
        .client
        .post(&app.url("/auth/register"))
        .json(&json!({
            "email": "valid@example.com",
            "name": "",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 422); // Validation error
}
