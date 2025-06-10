use reqwest::multipart::{Form, Part};

mod common;
use common::TestApp;

#[tokio::test]
async fn test_list_photos_unauthenticated() {
    let app = TestApp::new().await;
    let plant_id = uuid::Uuid::new_v4();

    let response = app
        .client
        .get(&app.url(&format!("/plants/{}/photos", plant_id)))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_list_photos_for_empty_plant() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "photos@example.com", "Photo User", "password123").await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Photo Plant", "Photographicus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // List photos
    let response = app
        .client
        .get(&app.url(&format!("/plants/{}/photos", plant_id)))
        .send()
        .await
        .expect("Failed to send list photos request");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["photos"].as_array().unwrap().len(), 0);
    assert_eq!(body["total"], 0);
}

#[tokio::test]
async fn test_upload_photo() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "upload@example.com", "Upload User", "password123").await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Upload Plant", "Uploadicus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Create fake image data (simulate a JPEG)
    let fake_image_data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header

    // Upload photo using multipart form
    let part = Part::bytes(fake_image_data.clone())
        .file_name("test-image.jpg")
        .mime_str("image/jpeg")
        .expect("Failed to create part");

    let form = Form::new().part("file", part);

    let response = app
        .client
        .post(&app.url(&format!("/plants/{}/photos", plant_id)))
        .multipart(form)
        .send()
        .await
        .expect("Failed to send upload photo request");

    assert_eq!(response.status(), 201);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(body["id"].is_string());
    assert_eq!(body["plantId"], plant_id);
    assert_eq!(body["originalFilename"], "test-image.jpg");
    assert_eq!(body["size"], fake_image_data.len());
    assert_eq!(body["contentType"], "image/jpeg");
    assert!(body["filename"].as_str().unwrap().contains(plant_id));
    assert!(body["createdAt"].is_string());
}

#[tokio::test]
async fn test_upload_photo_validation_errors() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(
        &app,
        "validation@example.com",
        "Validation User",
        "password123",
    )
    .await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Validation Plant", "Validicus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Test invalid content type
    let part = Part::bytes(vec![1, 2, 3, 4])
        .file_name("test.txt")
        .mime_str("text/plain")
        .expect("Failed to create part");

    let form = Form::new().part("file", part);

    let response = app
        .client
        .post(&app.url(&format!("/plants/{}/photos", plant_id)))
        .multipart(form)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 422);

    // Test file too large (11MB > 10MB limit)
    let large_data = vec![0u8; 11_000_000]; // 11MB
    let part = Part::bytes(large_data)
        .file_name("huge.jpg")
        .mime_str("image/jpeg")
        .expect("Failed to create part");

    let form = Form::new().part("file", part);

    let response = app
        .client
        .post(&app.url(&format!("/plants/{}/photos", plant_id)))
        .multipart(form)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 422);

    // Test no file provided
    let form = Form::new(); // Empty form

    let response = app
        .client
        .post(&app.url(&format!("/plants/{}/photos", plant_id)))
        .multipart(form)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 422);
}

#[tokio::test]
async fn test_upload_photo_for_nonexistent_plant() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(
        &app,
        "nonexistent@example.com",
        "Nonexistent User",
        "password123",
    )
    .await;

    let fake_plant_id = uuid::Uuid::new_v4();

    // Try to upload photo to nonexistent plant
    let part = Part::bytes(vec![0xFF, 0xD8, 0xFF, 0xE0]) // JPEG header
        .file_name("test.jpg")
        .mime_str("image/jpeg")
        .expect("Failed to create part");

    let form = Form::new().part("file", part);

    let response = app
        .client
        .post(&app.url(&format!("/plants/{}/photos", fake_plant_id)))
        .multipart(form)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_upload_photo_unauthenticated() {
    let app = TestApp::new().await;
    let plant_id = uuid::Uuid::new_v4();

    let part = Part::bytes(vec![0xFF, 0xD8, 0xFF, 0xE0]) // JPEG header
        .file_name("test.jpg")
        .mime_str("image/jpeg")
        .expect("Failed to create part");

    let form = Form::new().part("file", part);

    let response = app
        .client
        .post(&app.url(&format!("/plants/{}/photos", plant_id)))
        .multipart(form)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_delete_photo() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "delete@example.com", "Delete User", "password123").await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Delete Plant", "Deleticus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Upload a photo first
    let part = Part::bytes(vec![0xFF, 0xD8, 0xFF, 0xE0]) // JPEG header
        .file_name("to-delete.jpg")
        .mime_str("image/jpeg")
        .expect("Failed to create part");

    let form = Form::new().part("file", part);

    let upload_response = app
        .client
        .post(&app.url(&format!("/plants/{}/photos", plant_id)))
        .multipart(form)
        .send()
        .await
        .expect("Failed to upload photo");

    assert_eq!(upload_response.status(), 201);

    let upload_body: serde_json::Value = upload_response
        .json()
        .await
        .expect("Failed to parse upload response");
    let photo_id = upload_body["id"].as_str().unwrap();

    // Delete the photo
    let response = app
        .client
        .delete(&app.url(&format!("/plants/{}/photos/{}", plant_id, photo_id)))
        .send()
        .await
        .expect("Failed to send delete photo request");

    assert_eq!(response.status(), 204);

    // Verify photo is deleted by listing photos
    let list_response = app
        .client
        .get(&app.url(&format!("/plants/{}/photos", plant_id)))
        .send()
        .await
        .expect("Failed to list photos");

    assert_eq!(list_response.status(), 200);

    let list_body: serde_json::Value = list_response
        .json()
        .await
        .expect("Failed to parse list response");
    assert_eq!(list_body["photos"].as_array().unwrap().len(), 0);
    assert_eq!(list_body["total"], 0);
}

#[tokio::test]
async fn test_delete_nonexistent_photo() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(
        &app,
        "deletenonexistent@example.com",
        "Delete User",
        "password123",
    )
    .await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Delete Plant", "Deleticus").await;
    let plant_id = plant["id"].as_str().unwrap();
    let fake_photo_id = uuid::Uuid::new_v4();

    // Try to delete nonexistent photo
    let response = app
        .client
        .delete(&app.url(&format!("/plants/{}/photos/{}", plant_id, fake_photo_id)))
        .send()
        .await
        .expect("Failed to send delete photo request");

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_delete_photo_unauthenticated() {
    let app = TestApp::new().await;
    let plant_id = uuid::Uuid::new_v4();
    let photo_id = uuid::Uuid::new_v4();

    let response = app
        .client
        .delete(&app.url(&format!("/plants/{}/photos/{}", plant_id, photo_id)))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_user_isolation_photos() {
    let app = TestApp::new().await;

    // Create two users
    common::create_test_user(&app, "user1@example.com", "User 1", "password123").await;
    let user1_plant = common::create_test_plant(&app, "User 1 Plant", "User1icus").await;
    let user1_plant_id = user1_plant["id"].as_str().unwrap();

    // Upload photo as user1
    let part = Part::bytes(vec![0xFF, 0xD8, 0xFF, 0xE0]) // JPEG header
        .file_name("user1-photo.jpg")
        .mime_str("image/jpeg")
        .expect("Failed to create part");

    let form = Form::new().part("file", part);

    let upload_response = app
        .client
        .post(&app.url(&format!("/plants/{}/photos", user1_plant_id)))
        .multipart(form)
        .send()
        .await
        .expect("Failed to upload photo");

    assert_eq!(upload_response.status(), 201);
    let upload_body: serde_json::Value = upload_response
        .json()
        .await
        .expect("Failed to parse upload response");
    let photo_id = upload_body["id"].as_str().unwrap();

    // Logout user1 and login as user2
    app.client
        .post(&app.url("/auth/logout"))
        .send()
        .await
        .expect("Failed to logout");

    common::create_test_user(&app, "user2@example.com", "User 2", "password123").await;

    // User2 should not be able to list user1's photos
    let list_response = app
        .client
        .get(&app.url(&format!("/plants/{}/photos", user1_plant_id)))
        .send()
        .await
        .expect("Failed to list photos");

    assert_eq!(list_response.status(), 404); // Plant not found for user2

    // User2 should not be able to delete user1's photos
    let delete_response = app
        .client
        .delete(&app.url(&format!("/plants/{}/photos/{}", user1_plant_id, photo_id)))
        .send()
        .await
        .expect("Failed to delete photo");

    assert_eq!(delete_response.status(), 404); // Plant not found for user2
}

#[tokio::test]
async fn test_serve_photo() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(&app, "serve@example.com", "Serve User", "password123").await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Serve Plant", "Servicus").await;
    let plant_id = plant["id"].as_str().unwrap();

    // Create fake image data (simulate a JPEG)
    let fake_image_data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header

    // Upload photo using multipart form
    let part = Part::bytes(fake_image_data.clone())
        .file_name("serve-test.jpg")
        .mime_str("image/jpeg")
        .expect("Failed to create part");

    let form = Form::new().part("file", part);

    let upload_response = app
        .client
        .post(&app.url(&format!("/plants/{}/photos", plant_id)))
        .multipart(form)
        .send()
        .await
        .expect("Failed to send upload photo request");

    assert_eq!(upload_response.status(), 201);

    let upload_body: serde_json::Value = upload_response
        .json()
        .await
        .expect("Failed to parse upload response");
    let photo_id = upload_body["id"].as_str().unwrap();

    // Serve the photo
    let serve_response = app
        .client
        .get(&app.url(&format!("/plants/{}/photos/{}", plant_id, photo_id)))
        .send()
        .await
        .expect("Failed to send serve photo request");

    assert_eq!(serve_response.status(), 200);
    assert_eq!(
        serve_response.headers().get("content-type").unwrap(),
        "image/jpeg"
    );
    assert_eq!(
        serve_response.headers().get("content-length").unwrap(),
        &fake_image_data.len().to_string()
    );

    let served_data = serve_response
        .bytes()
        .await
        .expect("Failed to get photo data");
    assert_eq!(served_data.to_vec(), fake_image_data);
}

#[tokio::test]
async fn test_serve_nonexistent_photo() {
    let app = TestApp::new().await;

    // Register and login user
    common::create_test_user(
        &app,
        "servenotfound@example.com",
        "Serve User",
        "password123",
    )
    .await;

    // Create a plant
    let plant = common::create_test_plant(&app, "Serve Plant", "Servicus").await;
    let plant_id = plant["id"].as_str().unwrap();
    let fake_photo_id = uuid::Uuid::new_v4();

    // Try to serve nonexistent photo
    let response = app
        .client
        .get(&app.url(&format!("/plants/{}/photos/{}", plant_id, fake_photo_id)))
        .send()
        .await
        .expect("Failed to send serve photo request");

    assert_eq!(response.status(), 404);
}
