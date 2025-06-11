mod common;
use common::TestApp;

#[tokio::test]
async fn test_list_tracking_entries_unauthenticated() {
    let app = TestApp::new().await;
    let plant_id = uuid::Uuid::new_v4();

    let response = app
        .client
        .get(app.url(&format!("/plants/{}/entries", plant_id)))
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
        .get(app.url(&format!("/plants/{}/entries", plant_id)))
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
        .post(app.url(&format!("/plants/{}/entries", plant_id)))
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
        .get(app.url(&format!("/plants/{}", plant_id)))
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
        .post(app.url(&format!("/plants/{}/entries", plant_id)))
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
        .get(app.url(&format!("/plants/{}", plant_id)))
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
        .post(app.url(&format!("/plants/{}/entries", plant_id)))
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
        .get(app.url(&format!("/plants/{}", plant_id)))
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

#[tokio::test]
async fn test_create_note_entry() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "note@example.com", "Note User", "password123").await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Note Plant", "Noticus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Create note tracking entry
    let response = app
        .client
        .post(app.url(&format!("/plants/{}/entries", plant_id)))
        .json(&serde_json::json!({
            "entryType": "note",
            "timestamp": "2024-01-01T16:00:00Z",
            "notes": "Plant is looking healthy with new growth"
        }))
        .send()
        .await
        .expect("Failed to send note entry request");

    assert_eq!(response.status(), 201);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(body["id"].is_string());
    assert_eq!(body["entryType"], "note");
    assert_eq!(body["plantId"], plant_id);
    assert_eq!(body["notes"], "Plant is looking healthy with new growth");
}

#[tokio::test]
async fn test_create_note_entry_with_photo_ids() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "notephoto@example.com", "Note Photo User", "password123").await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Note Photo Plant", "Photicus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Create note tracking entry with photo IDs
    let photo_id1 = uuid::Uuid::new_v4();
    let photo_id2 = uuid::Uuid::new_v4();
    let response = app
        .client
        .post(app.url(&format!("/plants/{}/entries", plant_id)))
        .json(&serde_json::json!({
            "entryType": "note",
            "timestamp": "2024-01-01T16:00:00Z",
            "notes": "Growth documentation with photos",
            "photoIds": [photo_id1, photo_id2]
        }))
        .send()
        .await
        .expect("Failed to send note entry request");

    assert_eq!(response.status(), 201);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(body["id"].is_string());
    assert_eq!(body["entryType"], "note");
    assert_eq!(body["plantId"], plant_id);
    assert_eq!(body["notes"], "Growth documentation with photos");
    
    // Verify photo IDs are stored
    let photo_ids = body["photoIds"].as_array().unwrap();
    assert_eq!(photo_ids.len(), 2);
}

#[tokio::test]
async fn test_get_tracking_entry() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "get@example.com", "Get User", "password123").await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Get Plant", "Geticus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Create tracking entry
    let create_response = app
        .client
        .post(app.url(&format!("/plants/{}/entries", plant_id)))
        .json(&serde_json::json!({
            "entryType": "watering",
            "timestamp": "2024-01-01T12:00:00Z",
            "notes": "Test watering for get"
        }))
        .send()
        .await
        .expect("Failed to create tracking entry");

    assert_eq!(create_response.status(), 201);
    let created_entry: serde_json::Value = create_response.json().await.expect("Failed to parse create response");
    let entry_id = created_entry["id"].as_str().unwrap();

    // Get the tracking entry
    let get_response = app
        .client
        .get(app.url(&format!("/plants/{}/entries/{}", plant_id, entry_id)))
        .send()
        .await
        .expect("Failed to get tracking entry");

    assert_eq!(get_response.status(), 200);

    let retrieved_entry: serde_json::Value = get_response.json().await.expect("Failed to parse get response");
    assert_eq!(retrieved_entry["id"], entry_id);
    assert_eq!(retrieved_entry["entryType"], "watering");
    assert_eq!(retrieved_entry["plantId"], plant_id);
    assert_eq!(retrieved_entry["notes"], "Test watering for get");
}

#[tokio::test]
async fn test_get_tracking_entry_not_found() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "notfound@example.com", "Not Found User", "password123").await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Not Found Plant", "Notfoundicus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Try to get non-existent tracking entry
    let non_existent_id = uuid::Uuid::new_v4();
    let response = app
        .client
        .get(app.url(&format!("/plants/{}/entries/{}", plant_id, non_existent_id)))
        .send()
        .await
        .expect("Failed to send get request");

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_update_tracking_entry() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "update@example.com", "Update User", "password123").await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Update Plant", "Updateicus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Create tracking entry
    let create_response = app
        .client
        .post(app.url(&format!("/plants/{}/entries", plant_id)))
        .json(&serde_json::json!({
            "entryType": "note",
            "timestamp": "2024-01-01T12:00:00Z",
            "notes": "Original note"
        }))
        .send()
        .await
        .expect("Failed to create tracking entry");

    assert_eq!(create_response.status(), 201);
    let created_entry: serde_json::Value = create_response.json().await.expect("Failed to parse create response");
    let entry_id = created_entry["id"].as_str().unwrap();

    // Update the tracking entry
    let photo_id = uuid::Uuid::new_v4();
    let update_response = app
        .client
        .put(app.url(&format!("/plants/{}/entries/{}", plant_id, entry_id)))
        .json(&serde_json::json!({
            "notes": "Updated note with more details",
            "photoIds": [photo_id]
        }))
        .send()
        .await
        .expect("Failed to update tracking entry");

    assert_eq!(update_response.status(), 200);

    let updated_entry: serde_json::Value = update_response.json().await.expect("Failed to parse update response");
    assert_eq!(updated_entry["id"], entry_id);
    assert_eq!(updated_entry["notes"], "Updated note with more details");
    
    // Verify photo IDs are updated
    let photo_ids = updated_entry["photoIds"].as_array().unwrap();
    assert_eq!(photo_ids.len(), 1);
}

#[tokio::test]
async fn test_delete_tracking_entry() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "delete@example.com", "Delete User", "password123").await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Delete Plant", "Deleteicus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Create tracking entry
    let create_response = app
        .client
        .post(app.url(&format!("/plants/{}/entries", plant_id)))
        .json(&serde_json::json!({
            "entryType": "watering",
            "timestamp": "2024-01-01T12:00:00Z",
            "notes": "To be deleted"
        }))
        .send()
        .await
        .expect("Failed to create tracking entry");

    assert_eq!(create_response.status(), 201);
    let created_entry: serde_json::Value = create_response.json().await.expect("Failed to parse create response");
    let entry_id = created_entry["id"].as_str().unwrap();

    // Delete the tracking entry
    let delete_response = app
        .client
        .delete(app.url(&format!("/plants/{}/entries/{}", plant_id, entry_id)))
        .send()
        .await
        .expect("Failed to delete tracking entry");

    assert_eq!(delete_response.status(), 204);

    // Verify entry is deleted by trying to get it
    let get_response = app
        .client
        .get(app.url(&format!("/plants/{}/entries/{}", plant_id, entry_id)))
        .send()
        .await
        .expect("Failed to get deleted tracking entry");

    assert_eq!(get_response.status(), 404);
}

#[tokio::test]
async fn test_list_tracking_entries_with_various_types() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "listall@example.com", "List All User", "password123").await;

    // Create a plant
    let plant = common::create_test_plant(&app, "List All Plant", "Listicus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Create various types of tracking entries
    let entries = vec![
        serde_json::json!({
            "entryType": "watering",
            "timestamp": "2024-01-01T12:00:00Z",
            "notes": "Morning watering"
        }),
        serde_json::json!({
            "entryType": "fertilizing",
            "timestamp": "2024-01-02T13:00:00Z",
            "notes": "Weekly fertilizer"
        }),
        serde_json::json!({
            "entryType": "note",
            "timestamp": "2024-01-03T14:00:00Z",
            "notes": "New leaf spotted!"
        })
    ];

    // Create all entries
    for entry in entries {
        let response = app
            .client
            .post(app.url(&format!("/plants/{}/entries", plant_id)))
            .json(&entry)
            .send()
            .await
            .expect("Failed to create tracking entry");
        assert_eq!(response.status(), 201);
    }

    // List all tracking entries
    let list_response = app
        .client
        .get(app.url(&format!("/plants/{}/entries", plant_id)))
        .send()
        .await
        .expect("Failed to list tracking entries");

    assert_eq!(list_response.status(), 200);

    let body: serde_json::Value = list_response.json().await.expect("Failed to parse list response");
    let entries_list = body["entries"].as_array().unwrap();
    assert_eq!(entries_list.len(), 3);
    assert_eq!(body["total"], 3);

    // Verify entries are ordered by timestamp (newest first)
    let timestamps: Vec<&str> = entries_list
        .iter()
        .map(|e| e["timestamp"].as_str().unwrap())
        .collect();
    
    // Should be in descending order (newest first)
    assert_eq!(timestamps[0], "2024-01-03T14:00:00Z"); // note
    assert_eq!(timestamps[1], "2024-01-02T13:00:00Z"); // fertilizing
    assert_eq!(timestamps[2], "2024-01-01T12:00:00Z"); // watering
}
