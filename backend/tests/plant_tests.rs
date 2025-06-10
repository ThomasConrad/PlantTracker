use serde_json::json;
use uuid::Uuid;

mod common;
use common::TestApp;

#[tokio::test]
async fn test_create_plant_authenticated() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "plant@example.com", "Plant User", "password123").await;

    // Create a plant
    let response = app
        .client
        .post(&app.url("/plants"))
        .json(&json!({
            "name": "My Fiddle Leaf Fig",
            "genus": "Ficus",
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14,
            "customMetrics": []
        }))
        .send()
        .await
        .expect("Failed to send create plant request");

    assert_eq!(response.status(), 201);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["name"], "My Fiddle Leaf Fig");
    assert_eq!(body["genus"], "Ficus");
    assert_eq!(body["wateringIntervalDays"], 7);
    assert_eq!(body["fertilizingIntervalDays"], 14);
    assert!(body["id"].is_string());
    assert!(body["userId"].is_string());
}

#[tokio::test]
async fn test_create_plant_unauthenticated() {
    let app = TestApp::new().await;

    let response = app
        .client
        .post(&app.url("/plants"))
        .json(&json!({
            "name": "Unauthorized Plant",
            "genus": "Ficus",
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14,
            "customMetrics": []
        }))
        .send()
        .await
        .expect("Failed to send create plant request");

    assert_eq!(response.status(), 401); // Unauthorized
}

#[tokio::test]
async fn test_list_plants_authenticated() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "list@example.com", "List User", "password123").await;

    // Create a few plants
    common::create_test_plant(&app, "Plant 1", "Genus1").await;
    common::create_test_plant(&app, "Plant 2", "Genus2").await;

    // List plants
    let response = app
        .client
        .get(&app.url("/plants"))
        .send()
        .await
        .expect("Failed to send list plants request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["total"], 2);
    assert_eq!(body["plants"].as_array().unwrap().len(), 2);
    assert_eq!(body["limit"], 20);
    assert_eq!(body["offset"], 0);

    // Verify plant data
    let plants = body["plants"].as_array().unwrap();
    assert!(plants.iter().any(|p| p["name"] == "Plant 1"));
    assert!(plants.iter().any(|p| p["name"] == "Plant 2"));
}

#[tokio::test]
async fn test_list_plants_unauthenticated() {
    let app = TestApp::new().await;

    let response = app
        .client
        .get(&app.url("/plants"))
        .send()
        .await
        .expect("Failed to send list plants request");

    assert_eq!(response.status(), 401); // Unauthorized
}

#[tokio::test]
async fn test_get_plant_by_id() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "get@example.com", "Get User", "password123").await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Specific Plant", "Specific Genus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Get the plant by ID
    let response = app
        .client
        .get(&app.url(&format!("/plants/{}", plant_id)))
        .send()
        .await
        .expect("Failed to send get plant request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["id"], plant_id);
    assert_eq!(body["name"], "Specific Plant");
    assert_eq!(body["genus"], "Specific Genus");
}

#[tokio::test]
async fn test_get_nonexistent_plant() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "notfound@example.com", "NotFound User", "password123").await;

    let fake_id = Uuid::new_v4();
    let response = app
        .client
        .get(&app.url(&format!("/plants/{}", fake_id)))
        .send()
        .await
        .expect("Failed to send get plant request");

    assert_eq!(response.status(), 404); // Not found
}

#[tokio::test]
async fn test_update_plant() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "update@example.com", "Update User", "password123").await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Original Plant", "Original Genus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Update the plant
    let response = app
        .client
        .put(&app.url(&format!("/plants/{}", plant_id)))
        .json(&json!({
            "name": "Updated Plant",
            "genus": "Updated Genus",
            "wateringIntervalDays": 5,
            "fertilizingIntervalDays": 21
        }))
        .send()
        .await
        .expect("Failed to send update plant request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["id"], plant_id);
    assert_eq!(body["name"], "Updated Plant");
    assert_eq!(body["genus"], "Updated Genus");
    assert_eq!(body["wateringIntervalDays"], 5);
    assert_eq!(body["fertilizingIntervalDays"], 21);
}

#[tokio::test]
async fn test_delete_plant() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "delete@example.com", "Delete User", "password123").await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Plant to Delete", "Doomed Genus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Delete the plant
    let response = app
        .client
        .delete(&app.url(&format!("/plants/{}", plant_id)))
        .send()
        .await
        .expect("Failed to send delete plant request");

    assert_eq!(response.status(), 204); // No content

    // Verify plant is gone
    let get_response = app
        .client
        .get(&app.url(&format!("/plants/{}", plant_id)))
        .send()
        .await
        .expect("Failed to send get plant request");

    assert_eq!(get_response.status(), 404); // Not found
}

#[tokio::test]
async fn test_plant_validation() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(
        &app,
        "validation@example.com",
        "Validation User",
        "password123",
    )
    .await;

    // Test invalid watering interval (too high)
    let response = app
        .client
        .post(&app.url("/plants"))
        .json(&json!({
            "name": "Invalid Plant",
            "genus": "Invalid Genus",
            "wateringIntervalDays": 500, // Too high
            "fertilizingIntervalDays": 14,
            "customMetrics": []
        }))
        .send()
        .await
        .expect("Failed to send create plant request");

    assert_eq!(response.status(), 422); // Validation error

    // Test empty name
    let response = app
        .client
        .post(&app.url("/plants"))
        .json(&json!({
            "name": "",
            "genus": "Valid Genus",
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14,
            "customMetrics": []
        }))
        .send()
        .await
        .expect("Failed to send create plant request");

    assert_eq!(response.status(), 422); // Validation error
}

#[tokio::test]
async fn test_plant_search() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "search@example.com", "Search User", "password123").await;

    // Create plants with different names and genera
    common::create_test_plant(&app, "Fiddle Leaf Fig", "Ficus").await;
    common::create_test_plant(&app, "Snake Plant", "Sansevieria").await;
    common::create_test_plant(&app, "Rubber Plant", "Ficus").await;

    // Search by name
    let response = app
        .client
        .get(&app.url("/plants?search=Snake"))
        .send()
        .await
        .expect("Failed to send search request");

    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["total"], 1);
    assert_eq!(body["plants"][0]["name"], "Snake Plant");

    // Search by genus
    let response = app
        .client
        .get(&app.url("/plants?search=Ficus"))
        .send()
        .await
        .expect("Failed to send search request");

    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["total"], 2);
}

#[tokio::test]
async fn test_plant_pagination() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(
        &app,
        "pagination@example.com",
        "Pagination User",
        "password123",
    )
    .await;

    // Create multiple plants
    for i in 1..=25 {
        common::create_test_plant(&app, &format!("Plant {}", i), "TestGenus").await;
    }

    // Test first page
    let response = app
        .client
        .get(&app.url("/plants?limit=10&offset=0"))
        .send()
        .await
        .expect("Failed to send pagination request");

    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["total"], 25);
    assert_eq!(body["plants"].as_array().unwrap().len(), 10);
    assert_eq!(body["limit"], 10);
    assert_eq!(body["offset"], 0);

    // Test second page
    let response = app
        .client
        .get(&app.url("/plants?limit=10&offset=10"))
        .send()
        .await
        .expect("Failed to send pagination request");

    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["total"], 25);
    assert_eq!(body["plants"].as_array().unwrap().len(), 10);
    assert_eq!(body["limit"], 10);
    assert_eq!(body["offset"], 10);
}
