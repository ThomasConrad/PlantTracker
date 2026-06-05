//! Shared database traits and helpers for reducing boilerplate across the codebase.
//!
//! - `DbParse` — parse UUID/DateTime strings from SQLite with consistent error messages
//! - `RequireAffected` — chain `.require_affected("Entity {id}")` on query results
//! - `verify_plant_ownership` — one-call plant existence + user check

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::sqlite::SqliteQueryResult;
use uuid::Uuid;

use crate::database::DatabasePool;
use crate::utils::errors::AppError;

// ─────────────────────────────────────────────────────────────────────────────
// DbParse — consistent parsing of SQLite text fields into Rust types
// ─────────────────────────────────────────────────────────────────────────────

/// Helpers for parsing SQLite text columns into typed Rust values.
/// All methods produce `AppError::Internal` on failure with a descriptive message.
pub struct DbParse;

impl DbParse {
    /// Parse a UUID stored as a text column.
    pub fn uuid(s: &str) -> Result<Uuid, AppError> {
        Uuid::parse_str(s).map_err(|_| AppError::Internal {
            message: format!("Invalid UUID in database: {s}"),
        })
    }

    /// Parse a required DateTime<Utc> stored as RFC3339 text.
    pub fn datetime(s: &str) -> Result<DateTime<Utc>, AppError> {
        s.parse::<DateTime<Utc>>()
            .or_else(|_| {
                // Some columns store date-only values (e.g. "2024-06-01")
                NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
                    .map_err(|_| ())
            })
            .map_err(|_| AppError::Internal {
                message: format!("Invalid datetime in database: {s}"),
            })
    }

    /// Parse an optional DateTime<Utc> (None input → None output).
    pub fn optional_datetime(s: Option<&str>) -> Result<Option<DateTime<Utc>>, AppError> {
        match s {
            Some(v) => Self::datetime(v).map(Some),
            None => Ok(None),
        }
    }

    /// Parse an optional UUID (None input → None output).
    pub fn optional_uuid(s: Option<&str>) -> Result<Option<Uuid>, AppError> {
        match s {
            Some(v) => Self::uuid(v).map(Some),
            None => Ok(None),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RequireAffected — ergonomic check on rows_affected after DELETE/UPDATE
// ─────────────────────────────────────────────────────────────────────────────

/// Extension trait on `SqliteQueryResult` to turn zero-affected-rows into `AppError::NotFound`.
pub trait RequireAffected {
    /// Returns `Ok(())` if at least one row was affected, otherwise `Err(NotFound)`.
    fn require_affected(self, resource: impl Into<String>) -> Result<(), AppError>;
}

impl RequireAffected for SqliteQueryResult {
    fn require_affected(self, resource: impl Into<String>) -> Result<(), AppError> {
        if self.rows_affected() == 0 {
            Err(AppError::NotFound {
                resource: resource.into(),
            })
        } else {
            Ok(())
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Plant ownership verification
// ─────────────────────────────────────────────────────────────────────────────

/// Verify that a plant exists and belongs to the given user.
/// Returns `Ok(())` or `Err(AppError::NotFound)`.
pub async fn verify_plant_ownership(
    pool: &DatabasePool,
    plant_id: &Uuid,
    user_id: &str,
) -> Result<(), AppError> {
    let plant_id_str = plant_id.to_string();
    let exists = sqlx::query_scalar!(
        r#"SELECT 1 as "x!" FROM plants WHERE id = ? AND user_id = ?"#,
        plant_id_str,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    if exists.is_none() {
        return Err(AppError::NotFound {
            resource: format!("Plant with id {plant_id}"),
        });
    }
    Ok(())
}
