use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

lazy_static! {
    static ref CONTENT_TYPE_REGEX: Regex = Regex::new(r"^image/(jpeg|png|gif|webp|avif)$").unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Photo {
    pub id: Uuid,
    pub plant_id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub size: i64,
    pub content_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created_at: DateTime<Utc>,
}

impl Photo {
    /// Generate the URL for this photo
    pub fn url(&self) -> String {
        format!(
            "/api/v1/plants/{}/photos/{}?v={}",
            self.plant_id,
            self.id,
            self.created_at.timestamp()
        )
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PhotosResponse {
    pub photos: Vec<Photo>,
    pub total: i64,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UploadPhotoRequest {
    #[validate(length(min = 1, max = 255))]
    pub original_filename: String,
    #[validate(range(min = 1, max = 10485760))] // Max 10MB
    pub size: i64,
    #[validate(regex(path = "*CONTENT_TYPE_REGEX"))]
    pub content_type: String,
    pub data: Vec<u8>, // Raw image data
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProcessedImageInfo {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub content_type: String,
}
