use serde_json::json;

#[tokio::test]
async fn test_registration_validation() {
    // Test that the CreateUserRequest validation works correctly
    use planty_api::models::CreateUserRequest;
    use validator::Validate;

    // Valid request
    let valid_request = CreateUserRequest {
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        invite_code: Some("TEST123".to_string()),
    };
    
    assert!(valid_request.validate().is_ok());

    // Invalid email
    let invalid_email_request = CreateUserRequest {
        name: "Test User".to_string(),
        email: "invalid-email".to_string(),
        password: "password123".to_string(),
        invite_code: Some("TEST123".to_string()),
    };
    
    let validation_result = invalid_email_request.validate();
    assert!(validation_result.is_err());
    let errors = validation_result.unwrap_err();
    assert!(errors.field_errors().contains_key("email"));

    // Short password
    let short_password_request = CreateUserRequest {
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        password: "short".to_string(),
        invite_code: Some("TEST123".to_string()),
    };
    
    let validation_result = short_password_request.validate();
    assert!(validation_result.is_err());
    let errors = validation_result.unwrap_err();
    assert!(errors.field_errors().contains_key("password"));

    // Short name
    let short_name_request = CreateUserRequest {
        name: "A".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        invite_code: Some("TEST123".to_string()),
    };
    
    let validation_result = short_name_request.validate();
    assert!(validation_result.is_err());
    let errors = validation_result.unwrap_err();
    assert!(errors.field_errors().contains_key("name"));
}

#[tokio::test]
async fn test_json_serialization() {
    // Test that serialization and deserialization works correctly
    use planty_api::models::CreateUserRequest;

    let request = CreateUserRequest {
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        invite_code: Some("TEST123".to_string()),
    };

    // Serialize to JSON
    let json_string = serde_json::to_string(&request).unwrap();
    println!("Serialized JSON: {}", json_string);

    // Deserialize back
    let deserialized: CreateUserRequest = serde_json::from_str(&json_string).unwrap();
    assert_eq!(deserialized.name, "Test User");
    assert_eq!(deserialized.email, "test@example.com");
    assert_eq!(deserialized.password, "password123");
    assert_eq!(deserialized.invite_code, Some("TEST123".to_string()));

    // Test with missing invite_code
    let json_without_invite = json!({
        "name": "Test User",
        "email": "test@example.com",
        "password": "password123"
    });

    let result: Result<CreateUserRequest, _> = serde_json::from_value(json_without_invite);
    assert!(result.is_ok());
    let request = result.unwrap();
    assert_eq!(request.invite_code, None);

    // Test frontend format
    let frontend_json = json!({
        "name": "Test User",
        "email": "test@example.com", 
        "password": "password123",
        "invite_code": "TEST123"
    });

    let result: Result<CreateUserRequest, _> = serde_json::from_value(frontend_json);
    assert!(result.is_ok());
    let request = result.unwrap();
    assert_eq!(request.invite_code, Some("TEST123".to_string()));
}