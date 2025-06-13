use chrono::{DateTime, Duration, Utc};
use serde_json::Value;

use crate::database::google_oauth;
use crate::database::DatabasePool;
use crate::models::plant::PlantResponse;
use crate::models::google_oauth::GoogleOAuthToken;
use crate::utils::errors::{AppError, Result};

/// Configuration for Google Tasks API
#[derive(Debug, Clone)]
pub struct GoogleTasksConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl GoogleTasksConfig {
    pub fn from_env() -> Result<Self> {
        let client_id = std::env::var("GOOGLE_CLIENT_ID")
            .map_err(|_| AppError::Configuration {
                message: "GOOGLE_CLIENT_ID environment variable not set".to_string(),
            })?;
        
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .map_err(|_| AppError::Configuration {
                message: "GOOGLE_CLIENT_SECRET environment variable not set".to_string(),
            })?;
        
        let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI")
            .unwrap_or_else(|_| {
                let host_ip = std::env::var("HOST_IP").unwrap_or_else(|_| "localhost".to_string());
                format!("http://{}:3000/api/v1/google-tasks/callback", host_ip)
            });
        
        Ok(Self {
            client_id,
            client_secret,
            redirect_uri,
        })
    }
}

/// Create an HTTP client for Google Tasks API calls
async fn create_http_client() -> Result<reqwest::Client> {
    let client = reqwest::Client::new();
    Ok(client)
}

/// Generate Google OAuth authorization URL
pub fn generate_auth_url(config: &GoogleTasksConfig, state: &str) -> String {
    let scope = "https://www.googleapis.com/auth/tasks";
    
    format!(
        "https://accounts.google.com/o/oauth2/auth?\
         client_id={}&\
         redirect_uri={}&\
         scope={}&\
         response_type=code&\
         access_type=offline&\
         prompt=consent&\
         state={}",
        urlencoding::encode(&config.client_id),
        urlencoding::encode(&config.redirect_uri),
        urlencoding::encode(scope),
        urlencoding::encode(state)
    )
}

/// Exchange authorization code for access and refresh tokens
pub async fn exchange_code_for_tokens(
    config: &GoogleTasksConfig,
    code: &str,
) -> Result<(String, Option<String>, Option<DateTime<Utc>>)> {
    let client = reqwest::Client::new();
    
    let params = [
        ("client_id", &config.client_id),
        ("client_secret", &config.client_secret),
        ("code", &code.to_string()),
        ("grant_type", &"authorization_code".to_string()),
        ("redirect_uri", &config.redirect_uri),
    ];
    
    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to exchange code for tokens: {}", e);
            AppError::External {
                message: "Failed to communicate with Google OAuth".to_string(),
            }
        })?;
    
    let token_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| {
            tracing::error!("Failed to parse token response: {}", e);
            AppError::External {
                message: "Invalid response from Google OAuth".to_string(),
            }
        })?;
    
    // Check for error in response
    if let Some(error) = token_response.get("error") {
        let error_description = token_response
            .get("error_description")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");
        
        tracing::error!("OAuth error: {} - {}", error, error_description);
        return Err(AppError::Authentication {
            message: format!("OAuth error: {}", error_description),
        });
    }
    
    let access_token = token_response
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::External {
            message: "No access token in response".to_string(),
        })?
        .to_string();
    
    let refresh_token = token_response
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let expires_in = token_response
        .get("expires_in")
        .and_then(|v| v.as_i64());
    
    let expires_at = expires_in.map(|seconds| Utc::now() + Duration::seconds(seconds));
    
    tracing::info!("Successfully exchanged code for tokens");
    Ok((access_token, refresh_token, expires_at))
}

/// Refresh an access token using the refresh token
pub async fn refresh_access_token(
    config: &GoogleTasksConfig,
    refresh_token: &str,
) -> Result<(String, Option<DateTime<Utc>>)> {
    let client = reqwest::Client::new();
    
    let params = [
        ("client_id", &config.client_id),
        ("client_secret", &config.client_secret),
        ("refresh_token", &refresh_token.to_string()),
        ("grant_type", &"refresh_token".to_string()),
    ];
    
    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to refresh access token: {}", e);
            AppError::External {
                message: "Failed to refresh access token".to_string(),
            }
        })?;
    
    let token_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| {
            tracing::error!("Failed to parse refresh response: {}", e);
            AppError::External {
                message: "Invalid response from Google OAuth".to_string(),
            }
        })?;
    
    let access_token = token_response
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::External {
            message: "No access token in refresh response".to_string(),
        })?
        .to_string();
    
    let expires_in = token_response
        .get("expires_in")
        .and_then(|v| v.as_i64());
    
    let expires_at = expires_in.map(|seconds| Utc::now() + Duration::seconds(seconds));
    
    Ok((access_token, expires_at))
}

/// Ensure the user has a valid access token, refreshing if necessary
pub async fn ensure_valid_token(
    pool: &DatabasePool,
    user_id: &str,
    config: &GoogleTasksConfig,
) -> Result<GoogleOAuthToken> {
    let mut token = google_oauth::get_oauth_token(pool, user_id)
        .await?
        .ok_or_else(|| AppError::Authentication {
            message: "No Google Tasks connection found".to_string(),
        })?;
    
    // Check if token is expired or expires soon (within 5 minutes)
    let needs_refresh = if let Some(expires_at) = token.expires_at {
        expires_at < Utc::now() + Duration::minutes(5)
    } else {
        false
    };
    
    if needs_refresh {
        if let Some(refresh_token) = &token.refresh_token {
            tracing::info!("Refreshing access token for user: {}", user_id);
            
            let (new_access_token, new_expires_at) = 
                refresh_access_token(config, refresh_token).await?;
            
            // Update the token in the database
            google_oauth::update_access_token(pool, user_id, &new_access_token, new_expires_at)
                .await?;
            
            // Update our local token
            token.access_token = new_access_token;
            token.expires_at = new_expires_at;
            token.updated_at = Utc::now();
        } else {
            return Err(AppError::Authentication {
                message: "Token expired and no refresh token available".to_string(),
            });
        }
    }
    
    Ok(token)
}

/// Create a task for plant care using Google Tasks API
pub async fn create_plant_care_task(
    token: &GoogleOAuthToken,
    plant: &PlantResponse,
    task_type: &str, // "watering" or "fertilizing"
    due_time: DateTime<Utc>,
    base_url: &str,
    task_list_id: &str,
) -> Result<String> {
    let (title, notes) = match task_type {
        "watering" => (
            format!("💧 Water {}", plant.name),
            format!(
                "Time to water your {} ({}).\nWater every {} days.\n\nView plant details: {}/plants/{}",
                plant.name,
                plant.genus,
                plant.watering_interval_days,
                base_url,
                plant.id
            ),
        ),
        "fertilizing" => (
            format!("🌱 Fertilize {}", plant.name),
            format!(
                "Time to fertilize your {} ({}).\nFertilize every {} days.\n\nView plant details: {}/plants/{}",
                plant.name,
                plant.genus,
                plant.fertilizing_interval_days,
                base_url,
                plant.id
            ),
        ),
        _ => return Err(AppError::Internal {
            message: "Invalid task type".to_string(),
        }),
    };
    
    let client = create_http_client().await?;
    
    let task_data = serde_json::json!({
        "title": title,
        "notes": notes,
        "due": due_time.to_rfc3339(),
        "status": "needsAction"
    });
    
    let response = client
        .post(format!("https://tasks.googleapis.com/tasks/v1/lists/{}/tasks", task_list_id))
        .header("Authorization", format!("Bearer {}", token.access_token))
        .header("Content-Type", "application/json")
        .json(&task_data)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to create task: {}", e);
            AppError::External {
                message: "Failed to create Google Task".to_string(),
            }
        })?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        tracing::error!("Google Tasks API error: {}", error_text);
        return Err(AppError::External {
            message: "Google Tasks API request failed".to_string(),
        });
    }
    
    let result: Value = response.json().await.map_err(|e| {
        tracing::error!("Failed to parse Google Tasks response: {}", e);
        AppError::External {
            message: "Invalid response from Google Tasks".to_string(),
        }
    })?;
    
    let task_id = result["id"].as_str().ok_or_else(|| AppError::External {
        message: "No task ID returned from Google Tasks".to_string(),
    })?.to_string();
    
    tracing::info!("Created {} task for plant {}: {}", task_type, plant.name, task_id);
    Ok(task_id)
}

/// Get or create a task list for plant care
pub async fn get_or_create_plant_care_task_list(token: &GoogleOAuthToken) -> Result<String> {
    let client = create_http_client().await?;
    
    // First, try to find existing "Plant Care" task list
    let response = client
        .get("https://tasks.googleapis.com/tasks/v1/users/@me/lists")
        .header("Authorization", format!("Bearer {}", token.access_token))
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to get task lists: {}", e);
            AppError::External {
                message: "Failed to get Google Task lists".to_string(),
            }
        })?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        tracing::error!("Google Tasks API error: {}", error_text);
        return Err(AppError::External {
            message: "Google Tasks API request failed".to_string(),
        });
    }
    
    let result: Value = response.json().await.map_err(|e| {
        tracing::error!("Failed to parse Google Tasks response: {}", e);
        AppError::External {
            message: "Invalid response from Google Tasks".to_string(),
        }
    })?;
    
    // Look for existing "Plant Care" task list
    if let Some(items) = result["items"].as_array() {
        for item in items {
            if let Some(title) = item["title"].as_str() {
                if title == "Plant Care" {
                    if let Some(id) = item["id"].as_str() {
                        tracing::info!("Found existing Plant Care task list: {}", id);
                        return Ok(id.to_string());
                    }
                }
            }
        }
    }
    
    // Create new task list if not found
    let task_list_data = serde_json::json!({
        "title": "Plant Care"
    });
    
    let response = client
        .post("https://tasks.googleapis.com/tasks/v1/users/@me/lists")
        .header("Authorization", format!("Bearer {}", token.access_token))
        .header("Content-Type", "application/json")
        .json(&task_list_data)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to create task list: {}", e);
            AppError::External {
                message: "Failed to create Google Task list".to_string(),
            }
        })?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        tracing::error!("Google Tasks API error: {}", error_text);
        return Err(AppError::External {
            message: "Google Tasks API request failed".to_string(),
        });
    }
    
    let result: Value = response.json().await.map_err(|e| {
        tracing::error!("Failed to parse Google Tasks response: {}", e);
        AppError::External {
            message: "Invalid response from Google Tasks".to_string(),
        }
    })?;
    
    let task_list_id = result["id"].as_str().ok_or_else(|| AppError::External {
        message: "No task list ID returned from Google Tasks".to_string(),
    })?.to_string();
    
    tracing::info!("Created Plant Care task list: {}", task_list_id);
    Ok(task_list_id)
}

/// Generate a secure random state parameter for OAuth
pub fn generate_oauth_state() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    Utc::now().timestamp_nanos_opt().unwrap_or(0).hash(&mut hasher);
    format!("{:x}", hasher.finish())
}