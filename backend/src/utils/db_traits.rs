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

    /// Parse a JSON array of UUID strings from a text column.
    /// Returns None if input is None or the JSON is malformed.
    /// Invalid UUIDs within the array are silently skipped.
    pub fn uuid_list_json(s: Option<&str>) -> Option<Vec<Uuid>> {
        s.and_then(|json| {
            serde_json::from_str::<Vec<String>>(json).ok().map(|ids| {
                ids.iter()
                    .filter_map(|id| Uuid::parse_str(id).ok())
                    .collect()
            })
        })
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

// ─────────────────────────────────────────────────────────────────────────────
// UpdateBuilder — dynamic UPDATE queries with mixed types
// ─────────────────────────────────────────────────────────────────────────────

/// A type-erased value for dynamic SQL binding.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum DynValue {
    Text(String),
    OptText(Option<String>),
    Int(i64),
    Float(f64),
    Null,
}

impl From<String> for DynValue {
    fn from(v: String) -> Self {
        Self::Text(v)
    }
}
impl From<&str> for DynValue {
    fn from(v: &str) -> Self {
        Self::Text(v.to_string())
    }
}
impl From<i64> for DynValue {
    fn from(v: i64) -> Self {
        Self::Int(v)
    }
}
impl From<i32> for DynValue {
    fn from(v: i32) -> Self {
        Self::Int(v as i64)
    }
}
impl From<f64> for DynValue {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

/// Builder for dynamic UPDATE statements that handles mixed column types.
///
/// Usage:
/// ```ignore
/// UpdateBuilder::new("plants")
///     .set("name", &request.name)
///     .set("genus", &request.genus)
///     .set_null("archived_at")  // explicit NULL
///     .where_eq("id", &plant_id_str)
///     .where_eq("user_id", user_id)
///     .execute(pool)
///     .await?;
/// ```
pub struct UpdateBuilder {
    table: String,
    sets: Vec<(String, DynValue)>,
    wheres: Vec<(String, DynValue)>,
}

impl UpdateBuilder {
    pub fn new(table: &str) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            table: table.to_string(),
            sets: vec![("updated_at".to_string(), DynValue::Text(now))],
            wheres: Vec::new(),
        }
    }

    /// Conditionally set a column if the Option is Some.
    pub fn set_opt<T: Into<DynValue>>(mut self, column: &str, value: Option<T>) -> Self {
        if let Some(v) = value {
            self.sets.push((column.to_string(), v.into()));
        }
        self
    }

    /// Handle double-Option fields (common in PATCH semantics):
    /// - `None` → skip (field not provided)
    /// - `Some(None)` → set to NULL
    /// - `Some(Some(v))` → set to value
    pub fn set_nullable<T: Into<DynValue>>(self, column: &str, value: Option<Option<T>>) -> Self {
        match value {
            None => self,
            Some(None) => self.set_null(column),
            Some(Some(v)) => self.set(column, v),
        }
    }

    /// Always set a column to a value.
    pub fn set<T: Into<DynValue>>(mut self, column: &str, value: T) -> Self {
        self.sets.push((column.to_string(), value.into()));
        self
    }

    /// Set a column to NULL.
    pub fn set_null(mut self, column: &str) -> Self {
        self.sets.push((column.to_string(), DynValue::Null));
        self
    }

    /// Add a WHERE condition (all are AND-ed).
    pub fn where_eq(mut self, column: &str, value: &str) -> Self {
        self.wheres
            .push((column.to_string(), DynValue::Text(value.to_string())));
        self
    }

    /// Execute the UPDATE. Returns the number of rows affected.
    /// Returns Ok(0) if no SET clauses were added (besides updated_at).
    pub async fn execute(self, pool: &DatabasePool) -> Result<u64, AppError> {
        if self.sets.len() <= 1 {
            // Only updated_at was set — nothing to do
            return Ok(0);
        }

        let set_clause = self
            .sets
            .iter()
            .map(|(col, val)| match val {
                DynValue::Null => format!("{col} = NULL"),
                _ => format!("{col} = ?"),
            })
            .collect::<Vec<_>>()
            .join(", ");

        let where_clause = self
            .wheres
            .iter()
            .map(|(col, _)| format!("{col} = ?"))
            .collect::<Vec<_>>()
            .join(" AND ");

        let sql = format!(
            "UPDATE {} SET {} WHERE {}",
            self.table, set_clause, where_clause
        );

        let mut query = sqlx::query(&sql);

        // Bind SET values (skip NULLs since they're inlined)
        for (_, val) in &self.sets {
            query = bind_dyn(query, val);
        }

        // Bind WHERE values
        for (_, val) in &self.wheres {
            query = bind_dyn(query, val);
        }

        let result = query.execute(pool).await.map_err(AppError::Database)?;
        Ok(result.rows_affected())
    }
}

/// Bind a DynValue to a sqlx Query. Skips Null values (they're inlined in SQL).
fn bind_dyn<'q>(
    query: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    val: &'q DynValue,
) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
    match val {
        DynValue::Text(s) => query.bind(s.as_str()),
        DynValue::OptText(Some(s)) => query.bind(s.as_str()),
        DynValue::OptText(None) => query.bind(None::<&str>),
        DynValue::Int(i) => query.bind(*i),
        DynValue::Float(f) => query.bind(*f),
        DynValue::Null => query, // NULL is inlined in the SQL
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// LogDbError — log + convert sqlx errors in one step
// ─────────────────────────────────────────────────────────────────────────────

/// Extension trait that logs a database error with context before converting to `AppError::Database`.
///
/// Replaces the repeated pattern:
/// ```ignore
/// .await.map_err(|e| { tracing::error!("Failed to X: {}", e); AppError::Database(e) })?
/// ```
///
/// Usage:
/// ```ignore
/// sqlx::query!(...).execute(pool).await.log_db_err("delete plant")?;
/// ```
pub trait LogDbError<T> {
    fn log_db_err(self, context: &str) -> Result<T, AppError>;
}

impl<T> LogDbError<T> for Result<T, sqlx::Error> {
    fn log_db_err(self, context: &str) -> Result<T, AppError> {
        self.map_err(|e| {
            tracing::error!("Failed to {}: {}", context, e);
            AppError::Database(e)
        })
    }
}
