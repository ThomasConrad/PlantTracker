use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

lazy_static! {
    static ref CONTENT_TYPE_REGEX: Regex = Regex::new(r"^image/(jpeg|png|gif|webp)$").unwrap();
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
    pub thumbnail_width: Option<i32>,
    pub thumbnail_height: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PhotosResponse {
    pub photos: Vec<Photo>,
    pub total: i64,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PhotoWithThumbnail {
    pub id: Uuid,
    pub plant_id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub size: i64,
    pub content_type: String,
    pub thumbnail_width: Option<i32>,
    pub thumbnail_height: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub full_url: String,
    pub thumbnail_url: Option<String>,
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
    pub data: Vec<u8>, // Base64 decoded image data
    pub generate_thumbnail: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailInfo {
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}
