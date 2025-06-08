use serde_json::json;

mod common;
use common::TestApp;

#[tokio::test]
async fn test_user_can_only_see_own_plants() {
    let app = TestApp::new().await;

    // Create two users
    let user1 = common::create_test_user(&app, "user1@example.com", "User One", "password123").await;
    let user1_id = user1["user"]["id"].as_str().unwrap();

    // Create plants for user1
    let plant1 = common::create_test_plant(&app, "User1 Plant1", "Genus1").await;
    let plant2 = common::create_test_plant(&app, "User1 Plant2", "Genus2").await;

    // Logout user1 and create user2
    app.client.post(&app.url("/auth/logout")).send().await.unwrap();
    
    let user2 = common::create_test_user(&app, "user2@example.com", "User Two", "password123").await;
    let user2_id = user2["user"]["id"].as_str().unwrap();

    // Create plants for user2
    let plant3 = common::create_test_plant(&app, "User2 Plant1", "Genus3").await;

    // User2 should only see their own plant
    let response = app
        .client
        .get(&app.url("/plants"))
        .send()
        .await
        .expect("Failed to list plants");

    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    
    assert_eq!(body["total"], 1);
    assert_eq!(body["plants"].as_array().unwrap().len(), 1);
    assert_eq!(body["plants"][0]["name"], "User2 Plant1");
    assert_eq!(body["plants"][0]["user_id"], user2_id);

    // User2 should not be able to access user1's plants
    let plant1_id = plant1["id"].as_str().unwrap();
    let response = app
        .client
        .get(&app.url(&format!("/plants/{}", plant1_id)))
        .send()
        .await
        .expect("Failed to get plant");

    assert_eq!(response.status(), 404); // Not found (because it doesn't belong to user2)

    // Login as user1 again
    app.client.post(&app.url("/auth/logout")).send().await.unwrap();
    common::login_user(&app, "user1@example.com", "password123").await;

    // User1 should see their own plants
    let response = app
        .client
        .get(&app.url("/plants"))
        .send()
        .await
        .expect("Failed to list plants");

    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    
    assert_eq!(body["total"], 2);
    assert_eq!(body["plants"].as_array().unwrap().len(), 2);
    
    // Verify both plants belong to user1
    let plants = body["plants"].as_array().unwrap();
    for plant in plants {
        assert_eq!(plant["user_id"], user1_id);
    }

    // User1 should be able to access their own plants
    let response = app
        .client
        .get(&app.url(&format!("/plants/{}", plant1_id)))
        .send()
        .await
        .expect("Failed to get plant");

    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["id"], plant1_id);
    assert_eq!(body["user_id"], user1_id);
}

#[tokio::test]
async fn test_user_cannot_modify_other_users_plants() {
    let app = TestApp::new().await;

    // Create user1 and a plant
    common::create_test_user(&app, "owner@example.com", "Plant Owner", "password123").await;
    let plant = common::create_test_plant(&app, "Owner's Plant", "OwnerGenus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Logout and create user2
    app.client.post(&app.url("/auth/logout")).send().await.unwrap();
    common::create_test_user(&app, "hacker@example.com", "Plant Hacker", "password123").await;

    // User2 tries to update user1's plant
    let response = app
        .client
        .put(&app.url(&format!("/plants/{}", plant_id)))
        .json(&json!({
            "name": "Hacked Plant",
            "genus": "HackerGenus"
        }))
        .send()
        .await
        .expect("Failed to update plant");

    assert_eq!(response.status(), 404); // Not found (access denied)

    // User2 tries to delete user1's plant
    let response = app
        .client
        .delete(&app.url(&format!("/plants/{}", plant_id)))
        .send()
        .await
        .expect("Failed to delete plant");

    assert_eq!(response.status(), 404); // Not found (access denied)

    // Login as original owner to verify plant is unchanged
    app.client.post(&app.url("/auth/logout")).send().await.unwrap();
    common::login_user(&app, "owner@example.com", "password123").await;

    let response = app
        .client
        .get(&app.url(&format!("/plants/{}", plant_id)))
        .send()
        .await
        .expect("Failed to get plant");

    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["name"], "Owner's Plant"); // Should be unchanged
    assert_eq!(body["genus"], "OwnerGenus"); // Should be unchanged
}

#[tokio::test]
async fn test_concurrent_users_isolated_sessions() {
    let app = TestApp::new().await;

    // Create two HTTP clients with separate cookie jars
    let client1 = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .expect("Failed to create client1");

    let client2 = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .expect("Failed to create client2");

    // Register and login user1 with client1
    let _user1_register = client1
        .post(&app.url("/auth/register"))
        .json(&json!({
            "email": "concurrent1@example.com",
            "name": "Concurrent User 1",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register user1");

    // Register and login user2 with client2
    let _user2_register = client2
        .post(&app.url("/auth/register"))
        .json(&json!({
            "email": "concurrent2@example.com",
            "name": "Concurrent User 2",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to register user2");

    // Create plants with both clients
    let plant1_response = client1
        .post(&app.url("/plants"))
        .json(&json!({
            "name": "Client1 Plant",
            "genus": "Client1Genus",
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14,
            "customMetrics": []
        }))
        .send()
        .await
        .expect("Failed to create plant with client1");

    let plant2_response = client2
        .post(&app.url("/plants"))
        .json(&json!({
            "name": "Client2 Plant",
            "genus": "Client2Genus",
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14,
            "customMetrics": []
        }))
        .send()
        .await
        .expect("Failed to create plant with client2");

    assert_eq!(plant1_response.status(), 200);
    assert_eq!(plant2_response.status(), 200);

    // Verify each client only sees their own plants
    let client1_plants = client1
        .get(&app.url("/plants"))
        .send()
        .await
        .expect("Failed to list plants with client1")
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse client1 plants");

    let client2_plants = client2
        .get(&app.url("/plants"))
        .send()
        .await
        .expect("Failed to list plants with client2")
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse client2 plants");

    assert_eq!(client1_plants["total"], 1);
    assert_eq!(client1_plants["plants"][0]["name"], "Client1 Plant");

    assert_eq!(client2_plants["total"], 1);
    assert_eq!(client2_plants["plants"][0]["name"], "Client2 Plant");

    // Verify each client can access their own profile
    let client1_me = client1
        .get(&app.url("/auth/me"))
        .send()
        .await
        .expect("Failed to get client1 profile")
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse client1 profile");

    let client2_me = client2
        .get(&app.url("/auth/me"))
        .send()
        .await
        .expect("Failed to get client2 profile")
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse client2 profile");

    assert_eq!(client1_me["email"], "concurrent1@example.com");
    assert_eq!(client2_me["email"], "concurrent2@example.com");
}

#[tokio::test]
async fn test_session_isolation_after_logout() {
    let app = TestApp::new().await;

    // Create user and plant
    common::create_test_user(&app, "session@example.com", "Session User", "password123").await;
    let plant = common::create_test_plant(&app, "Session Plant", "SessionGenus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Verify authenticated access works
    let response = app
        .client
        .get(&app.url(&format!("/plants/{}", plant_id)))
        .send()
        .await
        .expect("Failed to get plant");
    assert_eq!(response.status(), 200);

    // Logout
    let logout_response = app
        .client
        .post(&app.url("/auth/logout"))
        .send()
        .await
        .expect("Failed to logout");
    assert_eq!(logout_response.status(), 204);

    // Verify access is denied after logout
    let response = app
        .client
        .get(&app.url(&format!("/plants/{}", plant_id)))
        .send()
        .await
        .expect("Failed to get plant after logout");
    assert_eq!(response.status(), 401); // Unauthorized

    // Verify listing is also denied
    let response = app
        .client
        .get(&app.url("/plants"))
        .send()
        .await
        .expect("Failed to list plants after logout");
    assert_eq!(response.status(), 401); // Unauthorized

    // Verify creating plants is denied
    let response = app
        .client
        .post(&app.url("/plants"))
        .json(&json!({
            "name": "Unauthorized Plant",
            "genus": "UnauthorizedGenus",
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14,
            "customMetrics": []
        }))
        .send()
        .await
        .expect("Failed to create plant after logout");
    assert_eq!(response.status(), 401); // Unauthorized
}