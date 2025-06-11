use chrono::{DateTime, Duration, Utc};
use google_calendar3::{
    api::{Event, EventDateTime},
    CalendarHub,
};
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use yup_oauth2::AccessToken;

use crate::database::google_oauth;
use crate::database::DatabasePool;
use crate::models::plant::PlantResponse;
use crate::models::google_oauth::GoogleOAuthToken;
use crate::utils::errors::{AppError, Result};

type HttpsClient = hyper::Client<HttpsConnector<HttpConnector>>;

/// Configuration for Google Calendar API
#[derive(Debug, Clone)]
pub struct GoogleCalendarConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl GoogleCalendarConfig {
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
            .unwrap_or_else(|_| "http://localhost:3000/api/v1/google-calendar/callback".to_string());
        
        Ok(Self {
            client_id,
            client_secret,
            redirect_uri,
        })
    }
}

/// Create a Google Calendar Hub with authenticated access
pub async fn create_calendar_hub(token: &GoogleOAuthToken) -> Result<CalendarHub<HttpsClient>> {
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_only()
        .enable_http1()
        .build();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);
    
    // Create access token from stored token
    let access_token = AccessToken {
        access_token: token.access_token.clone(),
        refresh_token: token.refresh_token.clone(),
        expires_in: token.expires_at.map(|exp| {
            let now = Utc::now();
            if exp > now {
                Some((exp - now).num_seconds())
            } else {
                Some(0)
            }
        }).flatten(),
        expires_in_timestamp: token.expires_at.map(|exp| exp.timestamp()),
        scope: Some(token.scope.clone()),
        token_type: Some(token.token_type.clone()),
    };
    
    // Create authenticator with the access token
    let auth = yup_oauth2::AccessTokenAuthenticator::builder(access_token)
        .build()
        .await?;
    
    let hub = CalendarHub::new(client, auth);
    Ok(hub)
}

/// Generate Google OAuth authorization URL
pub fn generate_auth_url(config: &GoogleCalendarConfig, state: &str) -> String {
    let scope = "https://www.googleapis.com/auth/calendar.events";
    
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
    config: &GoogleCalendarConfig,
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
    config: &GoogleCalendarConfig,
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
    config: &GoogleCalendarConfig,
) -> Result<GoogleOAuthToken> {
    let mut token = google_oauth::get_oauth_token(pool, user_id)
        .await?
        .ok_or_else(|| AppError::Authentication {
            message: "No Google Calendar connection found".to_string(),
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

/// Create a calendar event for plant care
pub async fn create_plant_care_event(
    hub: &CalendarHub<HttpsClient>,
    plant: &PlantResponse,
    event_type: &str, // "watering" or "fertilizing"
    due_time: DateTime<Utc>,
    base_url: &str,
) -> Result<String> {
    let (summary, description, emoji) = match event_type {
        "watering" => (
            format!("💧 Water {}", plant.name),
            format!(
                "Time to water your {} ({}). Water every {} days.\n\nView plant details: {}/plants/{}",
                plant.name,
                plant.genus,
                plant.watering_interval_days,
                base_url,
                plant.id
            ),
            "💧",
        ),
        "fertilizing" => (
            format!("🌱 Fertilize {}", plant.name),
            format!(
                "Time to fertilize your {} ({}). Fertilize every {} days.\n\nView plant details: {}/plants/{}",
                plant.name,
                plant.genus,
                plant.fertilizing_interval_days,
                base_url,
                plant.id
            ),
            "🌱",
        ),
        _ => return Err(AppError::Internal {
            message: "Invalid event type".to_string(),
        }),
    };
    
    let event = Event {
        summary: Some(summary),
        description: Some(description),
        start: Some(EventDateTime {
            date_time: Some(due_time.to_rfc3339()),
            time_zone: Some("UTC".to_string()),
            ..Default::default()
        }),
        end: Some(EventDateTime {
            date_time: Some((due_time + Duration::hours(1)).to_rfc3339()),
            time_zone: Some("UTC".to_string()),
            ..Default::default()
        }),
        location: Some(format!("Plant: {} ({})", plant.name, plant.genus)),
        ..Default::default()
    };
    
    let result = hub
        .events()
        .insert(event, "primary")
        .doit()
        .await
        .map_err(|e| {
            tracing::error!("Failed to create calendar event: {}", e);
            AppError::External {
                message: "Failed to create Google Calendar event".to_string(),
            }
        })?;
    
    let event_id = result.1.id.ok_or_else(|| AppError::External {
        message: "No event ID returned from Google Calendar".to_string(),
    })?;
    
    tracing::info!("Created {} event for plant {}: {}", event_type, plant.name, event_id);
    Ok(event_id)
}

/// Generate a secure random state parameter for OAuth
pub fn generate_oauth_state() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    Utc::now().timestamp_nanos_opt().unwrap_or(0).hash(&mut hasher);
    format!("{:x}", hasher.finish())
}