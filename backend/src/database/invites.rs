use chrono::Utc;
use uuid::Uuid;
use crate::database::DatabasePool;

use crate::models::{
    CreateInviteRequest, InviteCode, InviteCodeRow, WaitlistEntry, WaitlistEntryRow,
    WaitlistSignupRequest,
};
use crate::utils::errors::{AppError, Result};

pub async fn create_invite_code(
    pool: &DatabasePool,
    request: &CreateInviteRequest,
    created_by: Option<&str>,
) -> Result<InviteCode> {
    let id = Uuid::new_v4().to_string();
    let code = InviteCode::generate_code();
    let max_uses = request.max_uses.unwrap_or(1);
    let now = Utc::now();

    let expires_at_str = request.expires_at.map(|dt| dt.to_rfc3339());
    let now_str = now.to_rfc3339();

    let invite_row = sqlx::query_as::<_, InviteCodeRow>(
        r#"
        INSERT INTO invite_codes (id, code, created_by, max_uses, expires_at, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $6)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(&code)
    .bind(created_by)
    .bind(max_uses)
    .bind(expires_at_str)
    .bind(&now_str)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    let invite = invite_row.to_invite_code()?;

    Ok(invite)
}

pub async fn validate_invite_code(pool: &DatabasePool, code: &str) -> Result<InviteCode> {
    let invite_row = sqlx::query_as::<_, InviteCodeRow>(
        "SELECT * FROM invite_codes WHERE code = $1",
    )
    .bind(code)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?
    .ok_or(AppError::NotFound {
        resource: "Invite code".to_string(),
    })?;

    let invite = invite_row.to_invite_code()?;

    if !invite.is_valid() {
        return Err(AppError::Validation(
            validator::ValidationErrors::new(),
        ));
    }

    Ok(invite)
}

pub async fn use_invite_code(pool: &DatabasePool, code: &str, user_id: &str) -> Result<InviteCode> {
    let _invite = validate_invite_code(pool, code).await?;

    let now_str = Utc::now().to_rfc3339();

    let updated_invite_row = sqlx::query_as::<_, InviteCodeRow>(
        r#"
        UPDATE invite_codes 
        SET current_uses = current_uses + 1, 
            used_by = $2, 
            updated_at = $3
        WHERE code = $1
        RETURNING *
        "#,
    )
    .bind(code)
    .bind(user_id)
    .bind(&now_str)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    let updated_invite = updated_invite_row.to_invite_code()?;

    Ok(updated_invite)
}

pub async fn list_invite_codes(pool: &DatabasePool, created_by: Option<&str>) -> Result<Vec<InviteCode>> {
    let invite_rows = if let Some(user_id) = created_by {
        sqlx::query_as::<_, InviteCodeRow>(
            "SELECT * FROM invite_codes WHERE created_by = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?
    } else {
        sqlx::query_as::<_, InviteCodeRow>(
            "SELECT * FROM invite_codes ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?
    };

    let invites: Result<Vec<InviteCode>> = invite_rows
        .into_iter()
        .map(|row| row.to_invite_code())
        .collect();

    invites
}

pub async fn add_to_waitlist(
    pool: &DatabasePool,
    request: &WaitlistSignupRequest,
) -> Result<WaitlistEntry> {
    let id = Uuid::new_v4().to_string();
    let now_str = Utc::now().to_rfc3339();

    let entry_row = sqlx::query_as::<_, WaitlistEntryRow>(
        r#"
        INSERT INTO waitlist (id, email, name, message, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $5)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(&request.email)
    .bind(&request.name)
    .bind(&request.message)
    .bind(&now_str)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("unique constraint") {
            AppError::Validation({
                let mut errors = validator::ValidationErrors::new();
                errors.add(
                    "email",
                    validator::ValidationError::new("already_exists"),
                );
                errors
            })
        } else {
            AppError::Database(e)
        }
    })?;

    let entry = entry_row.to_waitlist_entry()?;

    Ok(entry)
}

pub async fn get_waitlist_entries(pool: &DatabasePool) -> Result<Vec<WaitlistEntry>> {
    let entry_rows = sqlx::query_as::<_, WaitlistEntryRow>(
        "SELECT * FROM waitlist ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let entries: Result<Vec<WaitlistEntry>> = entry_rows
        .into_iter()
        .map(|row| row.to_waitlist_entry())
        .collect();

    entries
}

pub async fn update_waitlist_status(
    pool: &DatabasePool,
    email: &str,
    status: &str,
    invite_code: Option<&str>,
) -> Result<WaitlistEntry> {
    let now = Utc::now();
    let now_str = now.to_rfc3339();
    let invited_at_str = if status == "invited" { Some(now_str.clone()) } else { None };

    let entry_row = sqlx::query_as::<_, WaitlistEntryRow>(
        r#"
        UPDATE waitlist 
        SET status = $2, invite_code = $3, invited_at = $4, updated_at = $5
        WHERE email = $1
        RETURNING *
        "#,
    )
    .bind(email)
    .bind(status)
    .bind(invite_code)
    .bind(invited_at_str)
    .bind(&now_str)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    let entry = entry_row.to_waitlist_entry()?;

    Ok(entry)
}