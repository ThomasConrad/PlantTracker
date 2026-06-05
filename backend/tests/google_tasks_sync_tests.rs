use chrono::Utc;
use reqwest::StatusCode;
use serde_json::{json, Value};
use serial_test::serial;
use std::sync::Once;
use wiremock::matchers::{header, method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod common;

use common::{create_test_plant, create_test_user, login_user, TestApp};
use planty_api::database::google_task_sync;

const TEST_PASSWORD: &str = "password123";

// Ensure Google env vars are set for all tests (safe for parallel since values are constant)
static INIT_ENV: Once = Once::new();
fn init_google_env() {
    INIT_ENV.call_once(|| {
        std::env::set_var("GOOGLE_CLIENT_ID", "test-client-id");
        std::env::set_var("GOOGLE_CLIENT_SECRET", "test-client-secret");
    });
}

/// Ergonomic test context with pre-authenticated user and plant
struct TestCtx {
    app: TestApp,
    user_id: String,
    plant_id: String,
    /// First care task ID (Water, interval=7)
    care_task_id: String,
    /// Second care task ID (Fertilize, interval=14)
    care_task_id_2: String,
}

impl TestCtx {
    /// Create a fully set-up test context: user registered, logged in, one plant with 2 care tasks
    async fn new(email: &str) -> Self {
        let app = TestApp::new().await;
        let user = create_test_user(&app, email, "Test User", TEST_PASSWORD).await;
        let user_id = user["user"]["id"].as_str().unwrap().to_string();
        login_user(&app, email, TEST_PASSWORD).await;
        let plant = create_test_plant(&app, "Test Plant", "Testus plantus").await;
        let plant_id = plant["id"].as_str().unwrap().to_string();
        let care_task_id = plant["careTasks"][0]["id"].as_str().unwrap().to_string();
        let care_task_id_2 = plant["careTasks"][1]["id"].as_str().unwrap().to_string();
        Self {
            app,
            user_id,
            plant_id,
            care_task_id,
            care_task_id_2,
        }
    }

    /// Store a fake Google OAuth token for this user
    async fn store_fake_token(&self) {
        use planty_api::database::google_oauth;
        let expires_at = Some(Utc::now() + chrono::Duration::hours(1));
        google_oauth::save_oauth_token(
            &self.app.db_pool,
            &self.user_id,
            "fake_access_token",
            Some("fake_refresh_token"),
            expires_at,
            "https://www.googleapis.com/auth/tasks",
        )
        .await
        .expect("Failed to store fake token");
    }

    /// Insert a sync record for this user's first care task
    async fn insert_sync(&self, google_task_id: &str, due_date: &str) {
        google_task_sync::insert_synced_task(
            &self.app.db_pool,
            &self.user_id,
            &self.plant_id,
            &self.care_task_id,
            google_task_id,
            "list-1",
            due_date,
        )
        .await
        .unwrap();
    }

    /// Insert a sync record for a specific care task
    async fn insert_sync_for(&self, care_task_id: &str, google_task_id: &str, due_date: &str) {
        google_task_sync::insert_synced_task(
            &self.app.db_pool,
            &self.user_id,
            &self.plant_id,
            care_task_id,
            google_task_id,
            "list-1",
            due_date,
        )
        .await
        .unwrap();
    }
}

// =============================================================================
// Database-level tests for google_task_sync
// =============================================================================

#[tokio::test]
async fn test_sync_insert_and_get() {
    let ctx = TestCtx::new("sync@test.com").await;

    // Initially no synced task
    let result = google_task_sync::get_synced_task(
        &ctx.app.db_pool,
        &ctx.user_id,
        &ctx.care_task_id,
        "2026-06-01",
    )
    .await
    .unwrap();
    assert!(result.is_none());

    // Insert
    ctx.insert_sync("google-task-abc", "2026-06-01").await;

    // Now it should exist
    let record = google_task_sync::get_synced_task(
        &ctx.app.db_pool,
        &ctx.user_id,
        &ctx.care_task_id,
        "2026-06-01",
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(record.google_task_id, "google-task-abc");
    assert_eq!(record.google_task_list_id, "list-1");
    assert_eq!(record.status, "needsAction");
    assert!(record.completed_at.is_none());
}

#[tokio::test]
async fn test_sync_deduplication() {
    let ctx = TestCtx::new("dedup@test.com").await;

    ctx.insert_sync("google-task-1", "2026-06-01").await;

    // Same user + care_task + due_date → UNIQUE violation
    let result = google_task_sync::insert_synced_task(
        &ctx.app.db_pool,
        &ctx.user_id,
        &ctx.plant_id,
        &ctx.care_task_id,
        "google-task-2",
        "list-1",
        "2026-06-01",
    )
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_sync_different_dates_allowed() {
    let ctx = TestCtx::new("dates@test.com").await;

    ctx.insert_sync("google-task-1", "2026-06-01").await;
    ctx.insert_sync("google-task-2", "2026-06-08").await;

    let r1 = google_task_sync::get_synced_task(
        &ctx.app.db_pool,
        &ctx.user_id,
        &ctx.care_task_id,
        "2026-06-01",
    )
    .await
    .unwrap();
    let r2 = google_task_sync::get_synced_task(
        &ctx.app.db_pool,
        &ctx.user_id,
        &ctx.care_task_id,
        "2026-06-08",
    )
    .await
    .unwrap();
    assert!(r1.is_some());
    assert!(r2.is_some());
}

#[tokio::test]
async fn test_get_incomplete_tasks() {
    let ctx = TestCtx::new("incomplete@test.com").await;

    ctx.insert_sync("gtask-1", "2026-06-01").await;
    ctx.insert_sync_for(&ctx.care_task_id_2.clone(), "gtask-2", "2026-06-01")
        .await;
    ctx.insert_sync("gtask-3", "2026-06-08").await;

    // All 3 should be incomplete
    let incomplete = google_task_sync::get_incomplete_tasks(&ctx.app.db_pool, &ctx.user_id)
        .await
        .unwrap();
    assert_eq!(incomplete.len(), 3);

    // Mark one as completed
    google_task_sync::mark_task_completed(&ctx.app.db_pool, "gtask-2")
        .await
        .unwrap();

    // Now only 2 incomplete
    let incomplete = google_task_sync::get_incomplete_tasks(&ctx.app.db_pool, &ctx.user_id)
        .await
        .unwrap();
    assert_eq!(incomplete.len(), 2);
    assert!(incomplete.iter().all(|r| r.google_task_id != "gtask-2"));
}

#[tokio::test]
async fn test_mark_task_completed() {
    let ctx = TestCtx::new("complete@test.com").await;

    ctx.insert_sync("google-task-x", "2026-06-15").await;

    google_task_sync::mark_task_completed(&ctx.app.db_pool, "google-task-x")
        .await
        .unwrap();

    let record = google_task_sync::get_synced_task(
        &ctx.app.db_pool,
        &ctx.user_id,
        &ctx.care_task_id,
        "2026-06-15",
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(record.status, "completed");
    assert!(record.completed_at.is_some());
}

#[tokio::test]
async fn test_delete_all_for_user() {
    let ctx = TestCtx::new("delete@test.com").await;

    for i in 1..=5 {
        ctx.insert_sync(&format!("gtask-{}", i), &format!("2026-07-{:02}", i))
            .await;
    }

    google_task_sync::delete_all_for_user(&ctx.app.db_pool, &ctx.user_id)
        .await
        .unwrap();

    let remaining = google_task_sync::get_incomplete_tasks(&ctx.app.db_pool, &ctx.user_id)
        .await
        .unwrap();
    assert_eq!(remaining.len(), 0);
}

#[tokio::test]
async fn test_cleanup_old_tasks() {
    let ctx = TestCtx::new("cleanup@test.com").await;

    // Old completed task
    ctx.insert_sync("gtask-old", "2025-01-01").await;
    google_task_sync::mark_task_completed(&ctx.app.db_pool, "gtask-old")
        .await
        .unwrap();

    // Recent incomplete task
    ctx.insert_sync_for(&ctx.care_task_id_2.clone(), "gtask-new", "2026-12-01")
        .await;

    // Cleanup completed tasks before 2026-06-01
    let deleted = google_task_sync::cleanup_old_tasks(&ctx.app.db_pool, "2026-06-01")
        .await
        .unwrap();
    assert_eq!(deleted, 1);

    // Recent task still exists
    let remaining = google_task_sync::get_incomplete_tasks(&ctx.app.db_pool, &ctx.user_id)
        .await
        .unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].google_task_id, "gtask-new");
}

#[tokio::test]
async fn test_user_isolation() {
    let app = TestApp::new().await;

    // Create two users
    let u1 = create_test_user(&app, "iso1@test.com", "User 1", TEST_PASSWORD).await;
    let u1_id = u1["user"]["id"].as_str().unwrap();
    login_user(&app, "iso1@test.com", TEST_PASSWORD).await;
    let p1 = create_test_plant(&app, "Plant A", "Genus A").await;
    let p1_id = p1["id"].as_str().unwrap();
    let ct1 = p1["careTasks"][0]["id"].as_str().unwrap();

    let u2 = create_test_user(&app, "iso2@test.com", "User 2", TEST_PASSWORD).await;
    let u2_id = u2["user"]["id"].as_str().unwrap();
    login_user(&app, "iso2@test.com", TEST_PASSWORD).await;
    let p2 = create_test_plant(&app, "Plant B", "Genus B").await;
    let p2_id = p2["id"].as_str().unwrap();
    let ct2 = p2["careTasks"][0]["id"].as_str().unwrap();

    // User1 syncs a task
    google_task_sync::insert_synced_task(
        &app.db_pool,
        u1_id,
        p1_id,
        ct1,
        "gtask-u1",
        "list-1",
        "2026-06-01",
    )
    .await
    .unwrap();

    // User2 should not see it
    let u2_tasks = google_task_sync::get_incomplete_tasks(&app.db_pool, u2_id)
        .await
        .unwrap();
    assert_eq!(u2_tasks.len(), 0);

    // User2 syncs their own task (same date is fine)
    google_task_sync::insert_synced_task(
        &app.db_pool,
        u2_id,
        p2_id,
        ct2,
        "gtask-u2",
        "list-2",
        "2026-06-01",
    )
    .await
    .unwrap();

    let u1_tasks = google_task_sync::get_incomplete_tasks(&app.db_pool, u1_id)
        .await
        .unwrap();
    let u2_tasks = google_task_sync::get_incomplete_tasks(&app.db_pool, u2_id)
        .await
        .unwrap();
    assert_eq!(u1_tasks.len(), 1);
    assert_eq!(u2_tasks.len(), 1);
    assert_eq!(u1_tasks[0].google_task_id, "gtask-u1");
    assert_eq!(u2_tasks[0].google_task_id, "gtask-u2");
}

// =============================================================================
// Integration tests with wiremock (mocking Google Tasks API)
// =============================================================================

#[tokio::test]
#[serial]
async fn test_sync_tasks_with_mock_google_api() {
    init_google_env();
    let mock_server = MockServer::start().await;

    std::env::set_var("GOOGLE_TASKS_API_BASE_URL", &mock_server.uri());

    let ctx = TestCtx::new("mocksync@test.com").await;
    ctx.store_fake_token().await;

    // Mock: GET task lists (return existing "Plant Care" list)
    Mock::given(method("GET"))
        .and(path("/tasks/v1/users/@me/lists"))
        .and(header("Authorization", "Bearer fake_access_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "kind": "tasks#taskLists",
            "items": [{ "id": "mock-list-id", "title": "Plant Care" }]
        })))
        .mount(&mock_server)
        .await;

    // Mock: POST create task
    Mock::given(method("POST"))
        .and(path_regex(r"/tasks/v1/lists/.+/tasks"))
        .and(header("Authorization", "Bearer fake_access_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "created-task-id-1",
            "title": "💧 Water Test Plant",
            "status": "needsAction"
        })))
        .mount(&mock_server)
        .await;

    // First sync — should create tasks
    let response = ctx
        .app
        .client
        .post(format!("{}/google-tasks/sync-tasks", ctx.app.address))
        .json(&json!({ "days_ahead": 30 }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    let tasks_created = body["tasks_created"].as_i64().unwrap();
    assert!(tasks_created > 0, "Should have created at least one task");
    assert_eq!(body["tasks_skipped"], 0);

    // Second sync — should skip all (dedup working)
    let response2 = ctx
        .app
        .client
        .post(format!("{}/google-tasks/sync-tasks", ctx.app.address))
        .json(&json!({ "days_ahead": 30 }))
        .send()
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::OK);
    let body2: Value = response2.json().await.unwrap();
    assert_eq!(body2["tasks_created"], 0);
    assert_eq!(body2["tasks_skipped"], tasks_created);
}

#[tokio::test]
#[serial]
async fn test_poll_completions_with_mock_google_api() {
    init_google_env();
    let mock_server = MockServer::start().await;

    std::env::set_var("GOOGLE_TASKS_API_BASE_URL", &mock_server.uri());

    let ctx = TestCtx::new("mockpoll@test.com").await;
    ctx.store_fake_token().await;

    // Pre-insert a synced task record
    google_task_sync::insert_synced_task(
        &ctx.app.db_pool,
        &ctx.user_id,
        &ctx.plant_id,
        &ctx.care_task_id,
        "google-task-poll-1",
        "mock-list-id",
        "2026-06-03",
    )
    .await
    .unwrap();

    // Mock: GET task status — return "completed"
    Mock::given(method("GET"))
        .and(path(
            "/tasks/v1/lists/mock-list-id/tasks/google-task-poll-1",
        ))
        .and(header("Authorization", "Bearer fake_access_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "google-task-poll-1",
            "status": "completed",
            "completed": "2026-06-03T10:00:00.000Z"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Poll
    let response = ctx
        .app
        .client
        .post(format!("{}/google-tasks/poll-completions", ctx.app.address))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["completed"], 1);
    assert_eq!(body["checked"], 1);

    // Verify sync record marked completed
    let record = google_task_sync::get_synced_task(
        &ctx.app.db_pool,
        &ctx.user_id,
        &ctx.care_task_id,
        "2026-06-03",
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(record.status, "completed");

    // Second poll — nothing left to check
    let response2 = ctx
        .app
        .client
        .post(format!("{}/google-tasks/poll-completions", ctx.app.address))
        .send()
        .await
        .unwrap();
    let body2: Value = response2.json().await.unwrap();
    assert_eq!(body2["completed"], 0);
    assert_eq!(body2["checked"], 0);
}

#[tokio::test]
#[serial]
async fn test_poll_completions_task_still_pending() {
    init_google_env();
    let mock_server = MockServer::start().await;

    std::env::set_var("GOOGLE_TASKS_API_BASE_URL", &mock_server.uri());

    let ctx = TestCtx::new("pending@test.com").await;
    ctx.store_fake_token().await;
    ctx.insert_sync("gtask-pending", "2026-06-03").await;

    // Mock: task still needsAction
    Mock::given(method("GET"))
        .and(path("/tasks/v1/lists/list-1/tasks/gtask-pending"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "gtask-pending",
            "status": "needsAction"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let response = ctx
        .app
        .client
        .post(format!("{}/google-tasks/poll-completions", ctx.app.address))
        .send()
        .await
        .unwrap();

    let body: Value = response.json().await.unwrap();
    assert_eq!(body["completed"], 0);
    assert_eq!(body["checked"], 1);

    // Record still needsAction
    let record = google_task_sync::get_synced_task(
        &ctx.app.db_pool,
        &ctx.user_id,
        &ctx.care_task_id,
        "2026-06-03",
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(record.status, "needsAction");
}

#[tokio::test]
async fn test_poll_completions_requires_auth() {
    let app = TestApp::new().await;

    let response = app
        .client
        .post(format!("{}/google-tasks/poll-completions", app.address))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn test_sync_creates_task_list_if_not_exists() {
    init_google_env();
    let mock_server = MockServer::start().await;

    std::env::set_var("GOOGLE_TASKS_API_BASE_URL", &mock_server.uri());

    let ctx = TestCtx::new("newlist@test.com").await;
    ctx.store_fake_token().await;

    // Mock: GET task lists — no "Plant Care" list
    Mock::given(method("GET"))
        .and(path("/tasks/v1/users/@me/lists"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "kind": "tasks#taskLists",
            "items": [{ "id": "other-list", "title": "Shopping" }]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock: POST create task list
    Mock::given(method("POST"))
        .and(path("/tasks/v1/users/@me/lists"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "new-plant-care-list-id",
            "title": "Plant Care"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Mock: POST create task
    Mock::given(method("POST"))
        .and(path_regex(r"/tasks/v1/lists/new-plant-care-list-id/tasks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "task-in-new-list",
            "status": "needsAction"
        })))
        .mount(&mock_server)
        .await;

    let response = ctx
        .app
        .client
        .post(format!("{}/google-tasks/sync-tasks", ctx.app.address))
        .json(&json!({ "days_ahead": 14 }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["tasks_created"].as_i64().unwrap() > 0);
}

#[tokio::test]
#[serial]
async fn test_poll_handles_google_api_error_gracefully() {
    init_google_env();
    let mock_server = MockServer::start().await;

    std::env::set_var("GOOGLE_TASKS_API_BASE_URL", &mock_server.uri());

    let ctx = TestCtx::new("apierr@test.com").await;
    ctx.store_fake_token().await;
    ctx.insert_sync("gtask-err", "2026-06-10").await;

    // Mock: Google returns 500
    Mock::given(method("GET"))
        .and(path("/tasks/v1/lists/list-1/tasks/gtask-err"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let response = ctx
        .app
        .client
        .post(format!("{}/google-tasks/poll-completions", ctx.app.address))
        .send()
        .await
        .unwrap();

    // Should still succeed (graceful degradation), just 0 completed
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["completed"], 0);
    assert_eq!(body["checked"], 1);

    // Task should remain needsAction
    let record = google_task_sync::get_synced_task(
        &ctx.app.db_pool,
        &ctx.user_id,
        &ctx.care_task_id,
        "2026-06-10",
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(record.status, "needsAction");
}
