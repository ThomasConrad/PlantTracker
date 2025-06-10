mod common;
use common::TestApp;

#[tokio::test]
async fn test_list_tracking_entries_unauthenticated() {
    let app = TestApp::new().await;
    let plant_id = uuid::Uuid::new_v4();

    let response = app
        .client
        .get(&app.url(&format!("/plants/{}/entries", plant_id)))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_list_tracking_entries_authenticated() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "tracking@example.com", "Tracking User", "password123").await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Tracking Plant", "Trackicus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // List tracking entries
    let response = app
        .client
        .get(&app.url(&format!("/plants/{}/entries", plant_id)))
        .send()
        .await
        .expect("Failed to send list tracking entries request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["entries"].as_array().unwrap().len(), 0);
    assert_eq!(body["total"], 0);
}

#[tokio::test]
async fn test_create_tracking_entry() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(
        &app,
        "create_tracking@example.com",
        "Create User",
        "password123",
    )
    .await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Create Tracking Plant", "Createicus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Create tracking entry
    let response = app
        .client
        .post(&app.url(&format!("/plants/{}/entries", plant_id)))
        .json(&serde_json::json!({
            "entryType": "watering",
            "timestamp": "2024-01-01T12:00:00Z",
            "notes": "Watered the plant"
        }))
        .send()
        .await
        .expect("Failed to send create tracking entry request");

    assert_eq!(response.status(), 201);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(body["id"].is_string());
    assert_eq!(body["entryType"], "watering");
    assert_eq!(body["plantId"], plant_id);
}

#[tokio::test]
async fn test_watering_updates_plant_last_watered() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "watering@example.com", "Watering User", "password123").await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Watering Plant", "Watericus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Verify plant initially has no last_watered date
    let get_response = app
        .client
        .get(&app.url(&format!("/plants/{}", plant_id)))
        .send()
        .await
        .expect("Failed to get plant");

    assert_eq!(get_response.status(), 200);
    let initial_plant: serde_json::Value =
        get_response.json().await.expect("Failed to parse plant");
    assert!(initial_plant["lastWatered"].is_null());

    // Create watering tracking entry
    let watering_time = "2024-01-01T12:00:00Z";
    let response = app
        .client
        .post(&app.url(&format!("/plants/{}/entries", plant_id)))
        .json(&serde_json::json!({
            "entryType": "watering",
            "timestamp": watering_time,
            "notes": "Test watering"
        }))
        .send()
        .await
        .expect("Failed to send watering entry request");

    assert_eq!(response.status(), 201);

    // Get plant again and verify last_watered is updated
    let get_response = app
        .client
        .get(&app.url(&format!("/plants/{}", plant_id)))
        .send()
        .await
        .expect("Failed to get plant after watering");

    assert_eq!(get_response.status(), 200);
    let updated_plant: serde_json::Value = get_response
        .json()
        .await
        .expect("Failed to parse updated plant");
    assert_eq!(updated_plant["lastWatered"], watering_time);
}

#[tokio::test]
async fn test_fertilizing_updates_plant_last_fertilized() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(
        &app,
        "fertilizing@example.com",
        "Fertilizing User",
        "password123",
    )
    .await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Fertilizing Plant", "Fertilicus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Create fertilizing tracking entry
    let fertilizing_time = "2024-01-01T14:00:00Z";
    let response = app
        .client
        .post(&app.url(&format!("/plants/{}/entries", plant_id)))
        .json(&serde_json::json!({
            "entryType": "fertilizing",
            "timestamp": fertilizing_time,
            "notes": "Test fertilizing"
        }))
        .send()
        .await
        .expect("Failed to send fertilizing entry request");

    assert_eq!(response.status(), 201);

    // Get plant and verify last_fertilized is updated
    let get_response = app
        .client
        .get(&app.url(&format!("/plants/{}", plant_id)))
        .send()
        .await
        .expect("Failed to get plant after fertilizing");

    assert_eq!(get_response.status(), 200);
    let updated_plant: serde_json::Value = get_response
        .json()
        .await
        .expect("Failed to parse updated plant");
    assert_eq!(updated_plant["lastFertilized"], fertilizing_time);
}
