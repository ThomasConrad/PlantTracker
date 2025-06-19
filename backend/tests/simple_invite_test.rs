use serde_json::json;

#[tokio::test]
async fn test_invite_registration_flow() {
    use planty_api::database::{create_pool_with_url, run_migrations};
    use planty_api::database::invites as db_invites;
    use planty_api::database::users as db_users;
    use planty_api::models::{CreateInviteRequest, CreateUserRequest, UserRole};

    // Create in-memory database
    let pool = create_pool_with_url("sqlite::memory:")
        .await
        .expect("Failed to create test database");

    // Run migrations
    run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    // Create an admin user first (directly in database)
    let admin_request = CreateUserRequest {
        name: "Admin User".to_string(),
        email: "admin@test.com".to_string(),
        password: "password123".to_string(),
        invite_code: None,
    };

    let admin_user = db_users::create_user_internal(
        &pool,
        &admin_request,
        UserRole::Admin,
        true,
        None,
    )
    .await
    .expect("Failed to create admin user");

    // Create an invite code
    let invite_request = CreateInviteRequest {
        max_uses: Some(1),
        expires_at: None,
    };

    let invite = db_invites::create_invite_code(&pool, &invite_request, Some(&admin_user.id))
        .await
        .expect("Failed to create invite code");

    println!("Created invite code: {}", invite.code);
    assert_eq!(invite.max_uses, 1);
    assert_eq!(invite.current_uses, 0);

    // Test invite validation
    let validated_invite = db_invites::validate_invite_code(&pool, &invite.code)
        .await
        .expect("Failed to validate invite code");

    assert!(validated_invite.is_valid());

    // Test user registration with invite
    let user_request = CreateUserRequest {
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        invite_code: Some(invite.code.clone()),
    };

    // Test validation passes
    assert!(user_request.validate().is_ok());

    // Test JSON serialization/deserialization (this is what might be failing)
    let json_str = serde_json::to_string(&user_request).unwrap();
    println!("Serialized request: {}", json_str);

    let deserialized: CreateUserRequest = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.name, "Test User");
    assert_eq!(deserialized.email, "test@example.com");
    assert_eq!(deserialized.invite_code, Some(invite.code.clone()));

    // Test the validation middleware would accept this
    use validator::Validate;
    assert!(deserialized.validate().is_ok());

    // Now try to create the user
    let new_user = db_users::create_user(&pool, &user_request)
        .await
        .expect("Failed to create user with invite");

    assert_eq!(new_user.email, "test@example.com");
    assert_eq!(new_user.name, "Test User");

    // Use the invite code
    let used_invite = db_invites::use_invite_code(&pool, &invite.code, &new_user.id)
        .await
        .expect("Failed to use invite code");

    assert_eq!(used_invite.current_uses, 1);

    // Try to validate the invite again (should fail since it's used up)
    let validation_result = db_invites::validate_invite_code(&pool, &invite.code).await;
    assert!(validation_result.is_err()); // Should return an error since invite is used up
}

#[tokio::test]
async fn test_frontend_json_format() {
    // Test the exact JSON format that the frontend would send
    use planty_api::models::CreateUserRequest;
    use validator::Validate;

    let frontend_json = json!({
        "name": "Test User",
        "email": "test@example.com",
        "password": "password123",
        "invite_code": "TEST123"
    });

    let parsed: CreateUserRequest = serde_json::from_value(frontend_json)
        .expect("Failed to parse frontend JSON");

    assert_eq!(parsed.name, "Test User");
    assert_eq!(parsed.email, "test@example.com");
    assert_eq!(parsed.password, "password123");
    assert_eq!(parsed.invite_code, Some("TEST123".to_string()));

    // Test validation
    assert!(parsed.validate().is_ok());

    // Test case with missing invite_code
    let json_no_invite = json!({
        "name": "Test User",
        "email": "test@example.com",
        "password": "password123"
    });

    let parsed_no_invite: CreateUserRequest = serde_json::from_value(json_no_invite)
        .expect("Failed to parse JSON without invite");

    assert_eq!(parsed_no_invite.invite_code, None);
    assert!(parsed_no_invite.validate().is_ok());

    // Test validation failures
    let invalid_email_json = json!({
        "name": "Test User",
        "email": "invalid-email",
        "password": "password123",
        "invite_code": "TEST123"
    });

    let invalid_email: CreateUserRequest = serde_json::from_value(invalid_email_json)
        .expect("Failed to parse JSON with invalid email");

    assert!(invalid_email.validate().is_err());

    let short_password_json = json!({
        "name": "Test User", 
        "email": "test@example.com",
        "password": "short",
        "invite_code": "TEST123"
    });

    let short_password: CreateUserRequest = serde_json::from_value(short_password_json)
        .expect("Failed to parse JSON with short password");

    assert!(short_password.validate().is_err());
}