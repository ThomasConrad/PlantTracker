use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InviteCode {
    pub id: String,
    pub code: String,
    pub created_by: Option<String>,
    pub used_by: Option<String>,
    pub max_uses: i32,
    pub current_uses: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct InviteCodeRow {
    pub id: String,
    pub code: String,
    pub created_by: Option<String>,
    pub used_by: Option<String>,
    pub max_uses: i32,
    pub current_uses: i32,
    pub expires_at: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WaitlistEntry {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub message: Option<String>,
    pub status: String,
    pub invited_at: Option<DateTime<Utc>>,
    pub invite_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct WaitlistEntryRow {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub message: Option<String>,
    pub status: String,
    pub invited_at: Option<String>,
    pub invite_code: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct CreateInviteRequest {
    pub max_uses: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct WaitlistSignupRequest {
    #[validate(email)]
    pub email: String,
    pub name: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct ValidateInviteRequest {
    #[validate(length(min = 1))]
    pub code: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InviteResponse {
    pub id: String,
    pub code: String,
    pub max_uses: i32,
    pub current_uses: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WaitlistResponse {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl InviteCode {
    pub fn generate_code() -> String {
        Uuid::new_v4().to_string().replace("-", "")[..12].to_uppercase()
    }

    pub fn is_valid(&self) -> bool {
        self.is_active
            && self.current_uses < self.max_uses
            && self.expires_at.is_none_or(|exp| exp > Utc::now())
    }
}

impl From<InviteCode> for InviteResponse {
    fn from(invite: InviteCode) -> Self {
        Self {
            id: invite.id,
            code: invite.code,
            max_uses: invite.max_uses,
            current_uses: invite.current_uses,
            expires_at: invite.expires_at,
            is_active: invite.is_active,
            created_at: invite.created_at,
        }
    }
}

impl InviteCodeRow {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_invite_code(self) -> Result<InviteCode, crate::utils::errors::AppError> {
        Ok(InviteCode {
            id: self.id,
            code: self.code,
            created_by: self.created_by,
            used_by: self.used_by,
            max_uses: self.max_uses,
            current_uses: self.current_uses,
            expires_at: if let Some(expires_str) = self.expires_at {
                Some(expires_str.parse::<DateTime<Utc>>().map_err(|_| {
                    crate::utils::errors::AppError::Internal {
                        message: "Invalid datetime in database".to_string(),
                    }
                })?)
            } else {
                None
            },
            is_active: self.is_active,
            created_at: self.created_at.parse::<DateTime<Utc>>().map_err(|_| {
                crate::utils::errors::AppError::Internal {
                    message: "Invalid datetime in database".to_string(),
                }
            })?,
            updated_at: self.updated_at.parse::<DateTime<Utc>>().map_err(|_| {
                crate::utils::errors::AppError::Internal {
                    message: "Invalid datetime in database".to_string(),
                }
            })?,
        })
    }
}

impl WaitlistEntryRow {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_waitlist_entry(self) -> Result<WaitlistEntry, crate::utils::errors::AppError> {
        Ok(WaitlistEntry {
            id: self.id,
            email: self.email,
            name: self.name,
            message: self.message,
            status: self.status,
            invited_at: if let Some(invited_str) = self.invited_at {
                Some(invited_str.parse::<DateTime<Utc>>().map_err(|_| {
                    crate::utils::errors::AppError::Internal {
                        message: "Invalid datetime in database".to_string(),
                    }
                })?)
            } else {
                None
            },
            invite_code: self.invite_code,
            created_at: self.created_at.parse::<DateTime<Utc>>().map_err(|_| {
                crate::utils::errors::AppError::Internal {
                    message: "Invalid datetime in database".to_string(),
                }
            })?,
            updated_at: self.updated_at.parse::<DateTime<Utc>>().map_err(|_| {
                crate::utils::errors::AppError::Internal {
                    message: "Invalid datetime in database".to_string(),
                }
            })?,
        })
    }
}

impl From<WaitlistEntry> for WaitlistResponse {
    fn from(entry: WaitlistEntry) -> Self {
        Self {
            id: entry.id,
            email: entry.email,
            name: entry.name,
            status: entry.status,
            created_at: entry.created_at,
        }
    }
}