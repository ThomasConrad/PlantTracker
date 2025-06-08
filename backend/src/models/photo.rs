use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Photo {
    pub id: Uuid,
    pub plant_id: Uuid,
    pub url: String,
    pub thumbnail_url: String,
    pub caption: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PhotosResponse {
    pub photos: Vec<Photo>,
    pub total: i64,
}
