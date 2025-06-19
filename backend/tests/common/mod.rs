use axum::Router;
use reqwest::Client;
use sqlx::SqlitePool;
use tempfile::TempDir;
use tokio::net::TcpListener;

use planty_api::app_state::AppState;
use planty_api::auth;
use planty_api::handlers::{auth as auth_handlers, google_tasks, plants, invites};

pub struct TestApp {
    pub address: String,
    pub db_pool: SqlitePool,
    pub client: Client,
    pub _temp_dir: TempDir,
}

impl TestApp {
    pub async fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        // Use in-memory SQLite database for tests
        let database_url = "sqlite::memory:".to_string();

        // Set up database using the centralized pool creation function
        let db_pool = planty_api::database::create_pool_with_url(&database_url)
            .await
            .expect("Failed to create test database pool");

        // Run migrations for tests
        planty_api::database::run_migrations(&db_pool)
            .await
            .expect("Failed to run migrations");

        // Set up authentication layers
        let (session_layer, auth_layer) = auth::create_auth_layers(db_pool.clone());

        // Create app state
        let app_state = AppState::new(db_pool.clone());

        // Build app
        let app = Router::new()
            .nest("/auth", auth_handlers::routes())
            .nest("/plants", plants::routes())
            .nest("/invites", invites::routes())
            .nest("/google-tasks", google_tasks::routes())
            .with_state(app_state)
            .layer(auth_layer)
            .layer(session_layer);

        // Start server
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to address");
        let address = listener.local_addr().expect("Failed to get local address");
        let server_url = format!("http://{}", address);

        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("Failed to start test server");
        });

        // Wait a bit for server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Create HTTP client with cookie jar for session management
        let client = Client::builder()
            .cookie_store(true)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            address: server_url,
            db_pool,
            client,
            _temp_dir: temp_dir,
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.address, path)
    }
}

// Test helpers
pub async fn create_test_user(
    app: &TestApp,
    email: &str,
    name: &str,
    password: &str,
) -> serde_json::Value {
    // First create an admin user directly in the database to create invites
    use planty_api::database::users as db_users;
    use planty_api::models::{CreateUserRequest, UserRole};
    use planty_api::database::invites as db_invites;

    // Create admin if it doesn't exist
    let admin_email = "test-admin@example.com";
    let admin_exists = sqlx::query_scalar!("SELECT id FROM users WHERE email = ?", admin_email)
        .fetch_optional(&app.db_pool)
        .await
        .unwrap()
        .is_some();

    if !admin_exists {
        let admin_request = CreateUserRequest {
            name: "Test Admin".to_string(),
            email: admin_email.to_string(),
            password: "admin123".to_string(),
            invite_code: None,
        };

        let _admin_user = db_users::create_user_internal(
            &app.db_pool,
            &admin_request,
            UserRole::Admin,
            true,
            None,
        )
        .await
        .expect("Failed to create admin user");
    }

    // Login as admin to create invite
    let login_response = app
        .client
        .post(app.url("/auth/login"))
        .json(&serde_json::json!({
            "email": admin_email,
            "password": "admin123"
        }))
        .send()
        .await
        .expect("Failed to login as admin");
    
    assert_eq!(login_response.status(), 200);

    // Create invite
    let invite_response = app
        .client
        .post(app.url("/invites/create"))
        .json(&serde_json::json!({
            "max_uses": 1
        }))
        .send()
        .await
        .expect("Failed to create invite");
    
    assert_eq!(invite_response.status(), 201);
    let invite_data: serde_json::Value = invite_response.json().await.unwrap();
    let invite_code = invite_data["code"].as_str().unwrap();

    // Now register the user with the invite
    let response = app
        .client
        .post(app.url("/auth/register"))
        .json(&serde_json::json!({
            "email": email,
            "name": name,
            "password": password,
            "invite_code": invite_code
        }))
        .send()
        .await
        .expect("Failed to send register request");

    assert_eq!(response.status(), 201);
    response
        .json()
        .await
        .expect("Failed to parse register response")
}

pub async fn login_user(app: &TestApp, email: &str, password: &str) -> serde_json::Value {
    let response = app
        .client
        .post(app.url("/auth/login"))
        .json(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(response.status(), 200);
    response
        .json()
        .await
        .expect("Failed to parse login response")
}

pub async fn create_test_plant(app: &TestApp, name: &str, genus: &str) -> serde_json::Value {
    let response = app
        .client
        .post(app.url("/plants"))
        .json(&serde_json::json!({
            "name": name,
            "genus": genus,
            "wateringSchedule": {
                "intervalDays": 7
            },
            "fertilizingSchedule": {
                "intervalDays": 14
            },
            "customMetrics": []
        }))
        .send()
        .await
        .expect("Failed to send create plant request");

    assert_eq!(response.status(), 201);
    response
        .json()
        .await
        .expect("Failed to parse create plant response")
}

/// Create valid test image data for testing
pub fn create_test_image_data(width: u32, height: u32) -> Vec<u8> {
    use image::{DynamicImage, ImageOutputFormat};
    use std::io::Cursor;

    let img = DynamicImage::new_rgb8(width, height);
    let mut jpeg_data = Vec::new();
    img.write_to(
        &mut Cursor::new(&mut jpeg_data),
        ImageOutputFormat::Jpeg(80),
    )
    .unwrap();
    jpeg_data
}
