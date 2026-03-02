use serde_json::{json, Value};

mod common;
use common::TestApp;

#[tokio::test]
async fn test_invite_creation_requires_auth() {
    let app = TestApp::new().await;

    // Try to create invite without authentication - should fail
    let response = app
        .client
        .post(app.url("/invites/create"))
        .json(&json!({
            "max_uses": 3
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_invite_creation_requires_permission() {
    let app = TestApp::new().await;

    use planty_api::database::users as db_users;
    use planty_api::models::{CreateUserRequest, UserRole};

    let user_request = CreateUserRequest {
        name: "Regular User".to_string(),
        email: "regular@test.com".to_string(),
        password: "password123".to_string(),
        invite_code: None,
    };

    let _user =
        db_users::create_user_internal(&app.db_pool, &user_request, UserRole::User, false, Some(0))
            .await
            .expect("Failed to create regular user");

    let login_response = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({
            "email": "regular@test.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to login regular user");
    assert_eq!(login_response.status(), 200);

    let create_response = app
        .client
        .post(app.url("/invites/create"))
        .json(&json!({
            "max_uses": 1
        }))
        .send()
        .await
        .expect("Failed to send invite create request");

    assert_eq!(create_response.status(), 403);
}

#[tokio::test]
async fn test_invite_creation_disallows_unlimited_quota_users() {
    let app = TestApp::new().await;

    use planty_api::database::users as db_users;
    use planty_api::models::{CreateUserRequest, UserRole};

    let user_request = CreateUserRequest {
        name: "Legacy Unlimited User".to_string(),
        email: "legacy@test.com".to_string(),
        password: "password123".to_string(),
        invite_code: None,
    };

    let _user =
        db_users::create_user_internal(&app.db_pool, &user_request, UserRole::Admin, true, None)
            .await
            .expect("Failed to create legacy user");

    let login_response = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({
            "email": "legacy@test.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to login legacy user");
    assert_eq!(login_response.status(), 200);

    let create_response = app
        .client
        .post(app.url("/invites/create"))
        .json(&json!({
            "max_uses": 1
        }))
        .send()
        .await
        .expect("Failed to send invite create request");

    assert_eq!(create_response.status(), 403);
}

#[tokio::test]
async fn test_invite_validation() {
    let app = TestApp::new().await;

    // First we need to create an admin user directly in the database
    // since we can't register without an invite
    use planty_api::database::users as db_users;
    use planty_api::models::{CreateUserRequest, UserRole};

    let admin_request = CreateUserRequest {
        name: "Admin User".to_string(),
        email: "admin@test.com".to_string(),
        password: "password123".to_string(),
        invite_code: None,
    };

    let _admin_user = db_users::create_user_internal(
        &app.db_pool,
        &admin_request,
        UserRole::Admin,
        true,
        Some(50),
    )
    .await
    .expect("Failed to create admin user");

    // Login as admin
    let login_response = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({
            "email": "admin@test.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(login_response.status(), 200);

    // Create an invite
    let create_response = app
        .client
        .post(app.url("/invites/create"))
        .json(&json!({
            "max_uses": 3
        }))
        .send()
        .await
        .expect("Failed to send create invite request");

    assert_eq!(create_response.status(), 201);

    let invite_data: Value = create_response
        .json()
        .await
        .expect("Failed to parse invite response");

    let invite_code = invite_data["code"].as_str().unwrap();
    assert!(!invite_code.is_empty());
    assert_eq!(invite_data["max_uses"], 3);
    assert_eq!(invite_data["current_uses"], 0);

    // Test invite validation
    let validate_response = app
        .client
        .post(app.url("/invites/validate"))
        .json(&json!({
            "code": invite_code
        }))
        .send()
        .await
        .expect("Failed to send validate request");

    assert_eq!(validate_response.status(), 200);

    let validate_data: Value = validate_response
        .json()
        .await
        .expect("Failed to parse validate response");

    assert_eq!(validate_data["valid"], true);
    assert_eq!(validate_data["uses_remaining"], 3);
}

#[tokio::test]
async fn test_invite_list() {
    let app = TestApp::new().await;

    // Create admin user
    use planty_api::database::users as db_users;
    use planty_api::models::{CreateUserRequest, UserRole};

    let admin_request = CreateUserRequest {
        name: "Admin User".to_string(),
        email: "admin@test.com".to_string(),
        password: "password123".to_string(),
        invite_code: None,
    };

    let _admin_user = db_users::create_user_internal(
        &app.db_pool,
        &admin_request,
        UserRole::Admin,
        true,
        Some(50),
    )
    .await
    .expect("Failed to create admin user");

    // Login as admin
    let _login_response = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({
            "email": "admin@test.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send login request");

    // Create an invite
    let _create_response = app
        .client
        .post(app.url("/invites/create"))
        .json(&json!({
            "max_uses": 1
        }))
        .send()
        .await
        .expect("Failed to send create invite request");

    // List invites
    let list_response = app
        .client
        .get(app.url("/invites/list"))
        .send()
        .await
        .expect("Failed to send list request");

    assert_eq!(list_response.status(), 200);

    let list_data: Value = list_response
        .json()
        .await
        .expect("Failed to parse list response");

    assert!(list_data["invites"].is_array());
    let invites = list_data["invites"].as_array().unwrap();
    assert_eq!(invites.len(), 1);
    assert_eq!(invites[0]["max_uses"], 1);
    assert_eq!(invites[0]["current_uses"], 0);
}

#[tokio::test]
async fn test_registration_with_invite() {
    let app = TestApp::new().await;

    // Create admin user
    use planty_api::database::users as db_users;
    use planty_api::models::{CreateUserRequest, UserRole};

    let admin_request = CreateUserRequest {
        name: "Admin User".to_string(),
        email: "admin@test.com".to_string(),
        password: "password123".to_string(),
        invite_code: None,
    };

    let _admin_user = db_users::create_user_internal(
        &app.db_pool,
        &admin_request,
        UserRole::Admin,
        true,
        Some(50),
    )
    .await
    .expect("Failed to create admin user");

    // Login as admin
    let _login_response = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({
            "email": "admin@test.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send login request");

    // Create an invite
    let create_response = app
        .client
        .post(app.url("/invites/create"))
        .json(&json!({
            "max_uses": 1
        }))
        .send()
        .await
        .expect("Failed to send create invite request");

    let invite_data: Value = create_response
        .json()
        .await
        .expect("Failed to parse invite response");

    let invite_code = invite_data["code"].as_str().unwrap();

    // Test registration with the invite code
    let register_response = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({
            "name": "New User",
            "email": "newuser@test.com",
            "password": "password123",
            "invite_code": invite_code
        }))
        .send()
        .await
        .expect("Failed to send register request");

    if register_response.status() != 201 {
        let error_text = register_response.text().await.unwrap();
        println!("Registration failed with: {}", error_text);
        panic!("Registration should succeed with valid invite code");
    }

    let auth_data: Value = register_response
        .json()
        .await
        .expect("Failed to parse register response");

    assert_eq!(auth_data["user"]["email"], "newuser@test.com");
    assert_eq!(auth_data["user"]["name"], "New User");
}

#[tokio::test]
async fn test_registration_without_invite() {
    let app = TestApp::new().await;

    // Test registration without invite code
    let register_response = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({
            "name": "New User",
            "email": "newuser@test.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send register request");

    assert_eq!(register_response.status(), 401);

    let error_data: Value = register_response
        .json()
        .await
        .expect("Failed to parse error response");

    assert!(error_data["message"]
        .as_str()
        .unwrap()
        .contains("invite code"));
}

#[tokio::test]
async fn test_registration_with_invalid_invite() {
    let app = TestApp::new().await;

    // Test registration with invalid invite code
    let register_response = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({
            "name": "New User",
            "email": "newuser@test.com",
            "password": "password123",
            "invite_code": "INVALID_CODE"
        }))
        .send()
        .await
        .expect("Failed to send register request");

    assert_eq!(register_response.status(), 401);

    let error_data: Value = register_response
        .json()
        .await
        .expect("Failed to parse error response");

    assert!(error_data["message"]
        .as_str()
        .unwrap()
        .contains("Invalid or expired"));
}

#[tokio::test]
async fn test_registration_validation_errors() {
    let app = TestApp::new().await;

    // Create admin and invite first
    use planty_api::database::users as db_users;
    use planty_api::models::{CreateUserRequest, UserRole};

    let admin_request = CreateUserRequest {
        name: "Admin User".to_string(),
        email: "admin@test.com".to_string(),
        password: "password123".to_string(),
        invite_code: None,
    };

    let _admin_user = db_users::create_user_internal(
        &app.db_pool,
        &admin_request,
        UserRole::Admin,
        true,
        Some(50),
    )
    .await
    .expect("Failed to create admin user");

    let _login_response = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({
            "email": "admin@test.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send login request");

    let create_response = app
        .client
        .post(app.url("/invites/create"))
        .json(&json!({
            "max_uses": 1
        }))
        .send()
        .await
        .expect("Failed to send create invite request");

    let invite_data: Value = create_response
        .json()
        .await
        .expect("Failed to parse invite response");

    let invite_code = invite_data["code"].as_str().unwrap();

    // Test registration with invalid email
    let register_response = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({
            "name": "Test User",
            "email": "invalid-email",
            "password": "password123",
            "invite_code": invite_code
        }))
        .send()
        .await
        .expect("Failed to send register request");

    assert_eq!(register_response.status(), 422); // Unprocessable Entity for validation errors

    let error_data: Value = register_response
        .json()
        .await
        .expect("Failed to parse error response");

    assert_eq!(error_data["error"], "validation_error");
    assert!(error_data["details"]["email"].is_array());

    // Test registration with short password
    let register_response = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({
            "name": "Test User",
            "email": "test@example.com",
            "password": "short",
            "invite_code": invite_code
        }))
        .send()
        .await
        .expect("Failed to send register request");

    assert_eq!(register_response.status(), 422);

    let error_data: Value = register_response
        .json()
        .await
        .expect("Failed to parse error response");

    assert_eq!(error_data["error"], "validation_error");
    assert!(error_data["details"]["password"].is_array());

    // Test registration with short name
    let register_response = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({
            "name": "A",
            "email": "test@example.com",
            "password": "password123",
            "invite_code": invite_code
        }))
        .send()
        .await
        .expect("Failed to send register request");

    assert_eq!(register_response.status(), 422);

    let error_data: Value = register_response
        .json()
        .await
        .expect("Failed to parse error response");

    assert_eq!(error_data["error"], "validation_error");
    assert!(error_data["details"]["name"].is_array());
}

#[tokio::test]
async fn test_invite_single_use_enforcement() {
    let app = TestApp::new().await;

    // Create admin user
    use planty_api::database::users as db_users;
    use planty_api::models::{CreateUserRequest, UserRole};

    let admin_request = CreateUserRequest {
        name: "Admin User".to_string(),
        email: "admin@test.com".to_string(),
        password: "password123".to_string(),
        invite_code: None,
    };

    let _admin_user = db_users::create_user_internal(
        &app.db_pool,
        &admin_request,
        UserRole::Admin,
        true,
        Some(50),
    )
    .await
    .expect("Failed to create admin user");

    // Login as admin
    let _login_response = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({
            "email": "admin@test.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to send login request");

    // Create a single-use invite
    let create_response = app
        .client
        .post(app.url("/invites/create"))
        .json(&json!({
            "max_uses": 1
        }))
        .send()
        .await
        .expect("Failed to send create invite request");

    let invite_data: Value = create_response
        .json()
        .await
        .expect("Failed to parse invite response");

    let invite_code = invite_data["code"].as_str().unwrap();

    // First registration should succeed
    let register_response1 = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({
            "name": "First User",
            "email": "first@test.com",
            "password": "password123",
            "invite_code": invite_code
        }))
        .send()
        .await
        .expect("Failed to send first register request");

    assert_eq!(register_response1.status(), 201);

    // Second registration with same invite should fail
    let register_response2 = app
        .client
        .post(app.url("/auth/register"))
        .json(&json!({
            "name": "Second User",
            "email": "second@test.com",
            "password": "password123",
            "invite_code": invite_code
        }))
        .send()
        .await
        .expect("Failed to send second register request");

    assert_eq!(register_response2.status(), 401);

    let error_data: Value = register_response2
        .json()
        .await
        .expect("Failed to parse error response");

    assert!(error_data["message"]
        .as_str()
        .unwrap()
        .contains("Invalid or expired"));

    // Verify invite usage count increased
    let list_response = app
        .client
        .get(app.url("/invites/list"))
        .send()
        .await
        .expect("Failed to send list request");

    let list_data: Value = list_response
        .json()
        .await
        .expect("Failed to parse list response");

    let invites = list_data["invites"].as_array().unwrap();
    let used_invite = invites.iter().find(|inv| inv["code"] == invite_code);

    match used_invite {
        Some(invite) => {
            assert_eq!(invite["current_uses"], 1);
            assert_eq!(invite["max_uses"], 1);
        }
        None => {
            // If the invite is not in the list, it means it was removed or filtered out
            // which is acceptable behavior for fully consumed invites
            println!("Invite was removed from list after being fully consumed");
        }
    }
}

#[tokio::test]
async fn test_admin_can_invite_waitlist_entry() {
    std::env::set_var("WAITLIST_ENABLED", "true");
    let app = TestApp::new().await;

    use planty_api::database::users as db_users;
    use planty_api::models::{CreateUserRequest, UserRole};

    let admin_request = CreateUserRequest {
        name: "Admin User".to_string(),
        email: "admin@test.com".to_string(),
        password: "password123".to_string(),
        invite_code: None,
    };

    let _admin_user = db_users::create_user_internal(
        &app.db_pool,
        &admin_request,
        UserRole::Admin,
        true,
        Some(50),
    )
    .await
    .expect("Failed to create admin user");

    let login_response = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({
            "email": "admin@test.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to login as admin");
    assert_eq!(login_response.status(), 200);

    let waitlist_response = app
        .client
        .post(app.url("/invites/waitlist"))
        .json(&json!({
            "email": "waitlist-user@test.com",
            "name": "Waitlist User",
            "message": "Please invite me"
        }))
        .send()
        .await
        .expect("Failed to create waitlist entry");
    assert_eq!(waitlist_response.status(), 201);

    let created_waitlist: Value = waitlist_response
        .json()
        .await
        .expect("Failed to parse waitlist response");
    let waitlist_id = created_waitlist["id"]
        .as_str()
        .expect("Waitlist response should include id");

    let invite_response = app
        .client
        .post(app.url(&format!("/invites/waitlist/{waitlist_id}/invite")))
        .json(&json!({
            "max_uses": 2
        }))
        .send()
        .await
        .expect("Failed to invite waitlist entry");
    assert_eq!(invite_response.status(), 200);

    let invited_payload: Value = invite_response
        .json()
        .await
        .expect("Failed to parse invite waitlist response");

    assert!(invited_payload["invite_code"].as_str().is_some());
    assert_eq!(invited_payload["waitlist_entry"]["status"], "invited");
}

#[tokio::test]
async fn test_non_admin_cannot_invite_waitlist_entry() {
    std::env::set_var("WAITLIST_ENABLED", "true");
    let app = TestApp::new().await;

    use planty_api::database::users as db_users;
    use planty_api::models::{CreateUserRequest, UserRole};

    let user_request = CreateUserRequest {
        name: "Normal User".to_string(),
        email: "user@test.com".to_string(),
        password: "password123".to_string(),
        invite_code: None,
    };

    let _user =
        db_users::create_user_internal(&app.db_pool, &user_request, UserRole::User, false, Some(0))
            .await
            .expect("Failed to create normal user");

    let login_response = app
        .client
        .post(app.url("/auth/login"))
        .json(&json!({
            "email": "user@test.com",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to login as normal user");
    assert_eq!(login_response.status(), 200);

    // Create waitlist entry (public-ish endpoint)
    let waitlist_response = app
        .client
        .post(app.url("/invites/waitlist"))
        .json(&json!({
            "email": "another-waitlist@test.com"
        }))
        .send()
        .await
        .expect("Failed to create waitlist entry");
    assert_eq!(waitlist_response.status(), 201);

    let created_waitlist: Value = waitlist_response
        .json()
        .await
        .expect("Failed to parse waitlist response");
    let waitlist_id = created_waitlist["id"]
        .as_str()
        .expect("Waitlist response should include id");

    let invite_response = app
        .client
        .post(app.url(&format!("/invites/waitlist/{waitlist_id}/invite")))
        .json(&json!({}))
        .send()
        .await
        .expect("Failed to call invite endpoint");

    assert_eq!(invite_response.status(), 403);
}
