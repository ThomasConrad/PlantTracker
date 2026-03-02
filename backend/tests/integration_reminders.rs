use serde_json::json;

mod common;
use common::TestApp;

#[tokio::test]
async fn test_reminder_preferences_requires_auth() {
    let app = TestApp::new().await;

    let response = app
        .client
        .get(app.url("/reminders/preferences"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_update_and_fetch_reminder_preferences() {
    let app = TestApp::new().await;
    let user =
        common::create_test_user(&app, "reminders@test.com", "Reminder User", "password123").await;
    assert_eq!(user["user"]["email"], "reminders@test.com");

    let update = app
        .client
        .put(app.url("/reminders/preferences"))
        .json(&json!({
            "enabled": true,
            "reminderTime": "08:30",
            "timezone": "America/New_York",
            "browserNotificationsEnabled": true
        }))
        .send()
        .await
        .expect("Failed to update reminder preferences");

    assert_eq!(update.status(), 200);

    let get = app
        .client
        .get(app.url("/reminders/preferences"))
        .send()
        .await
        .expect("Failed to fetch reminder preferences");

    assert_eq!(get.status(), 200);
    let payload: serde_json::Value = get.json().await.expect("Failed to parse response");
    assert_eq!(payload["enabled"], true);
    assert_eq!(payload["reminderTime"], "08:30");
    assert_eq!(payload["timezone"], "America/New_York");
    assert_eq!(payload["browserNotificationsEnabled"], true);
}

#[tokio::test]
async fn test_dispatch_reminders_is_idempotent_for_same_due_date() {
    let app = TestApp::new().await;
    let _user =
        common::create_test_user(&app, "dispatch@test.com", "Dispatch User", "password123").await;

    let _prefs = app
        .client
        .put(app.url("/reminders/preferences"))
        .json(&json!({
            "enabled": true,
            "reminderTime": "00:00",
            "timezone": "UTC",
            "browserNotificationsEnabled": true
        }))
        .send()
        .await
        .expect("Failed to update reminder preferences");

    let create = app
        .client
        .post(app.url("/plants"))
        .json(&json!({
            "name": "Aloe",
            "genus": "Aloe",
            "wateringSchedule": { "intervalDays": 1 },
            "fertilizingSchedule": { "intervalDays": 30 },
            "lastWatered": "2020-01-01T00:00:00Z",
            "customMetrics": []
        }))
        .send()
        .await
        .expect("Failed to create plant");
    assert_eq!(create.status(), 201);

    let first_dispatch = app
        .client
        .post(app.url("/reminders/dispatch"))
        .send()
        .await
        .expect("Failed first dispatch");
    assert_eq!(first_dispatch.status(), 200);
    let first_payload: serde_json::Value = first_dispatch
        .json()
        .await
        .expect("Failed to parse first dispatch");
    assert!(first_payload["sentCount"].as_u64().unwrap_or(0) >= 1);

    let second_dispatch = app
        .client
        .post(app.url("/reminders/dispatch"))
        .send()
        .await
        .expect("Failed second dispatch");
    assert_eq!(second_dispatch.status(), 200);
    let second_payload: serde_json::Value = second_dispatch
        .json()
        .await
        .expect("Failed to parse second dispatch");
    assert_eq!(second_payload["sentCount"], 0);
}
