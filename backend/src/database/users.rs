use anyhow::Result;
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::Utc;
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;
use validator::{ValidationError, ValidationErrors};

// Argon2id configuration following OWASP recommendations:
// - 19 MiB of memory (19456 KB)
// - 2 iterations
// - 1 degree of parallelism
fn get_argon2_config() -> Argon2<'static> {
    Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(19456, 2, 1, None).unwrap(),
    )
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = get_argon2_config();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal {
            message: format!("Failed to hash password: {e}"),
        })
        .map(|h| h.to_string())
}

fn duplicate_email_validation_error() -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let mut email_error = ValidationError::new("email_already_exists");
    email_error.message = Some("An account with this email already exists".into());
    errors.add("email", email_error);
    errors
}

// For tests, use faster but still secure parameters
#[cfg(test)]
fn get_argon2_test_config() -> Argon2<'static> {
    Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(4096, 1, 1, None).unwrap(), // Faster for tests
    )
}

use crate::database::DatabasePool;
use crate::models::{CreateUserRequest, FirstDayOfWeek, PreferredUnits, User, UserRole, UserRow};
use crate::utils::errors::AppError;

pub async fn create_user(
    pool: &DatabasePool,
    request: &CreateUserRequest,
) -> Result<User, AppError> {
    // Get default invite limit from settings
    let default_limit = get_default_invite_limit(pool).await?;

    create_user_internal(pool, request, UserRole::User, false, Some(default_limit)).await
}

pub async fn create_user_internal(
    pool: &DatabasePool,
    request: &CreateUserRequest,
    role: UserRole,
    can_create_invites: bool,
    max_invites: Option<i32>,
) -> Result<User, AppError> {
    // Check if user with this email already exists
    if get_user_by_email(pool, &request.email).await.is_ok() {
        return Err(AppError::Validation(duplicate_email_validation_error()));
    }

    // Check total user limit
    let total_users = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

    let max_total_users = get_max_total_users(pool).await?;

    if total_users >= max_total_users {
        return Err(AppError::Internal {
            message: "Maximum number of users reached".to_string(),
        });
    }

    let user_id = Uuid::new_v4().to_string();

    // Generate salt and hash password with Argon2id
    let password_hash = hash_password(&request.password)?;

    let now = Utc::now().to_rfc3339();
    let role_str = role.to_string();

    let result = sqlx::query(
        r#"
        INSERT INTO users (id, email, name, first_day_of_week, preferred_units, password_hash, role, can_create_invites, max_invites, invites_created, created_at, updated_at)
        VALUES (?, ?, ?, 'sunday', 'metric', ?, ?, ?, ?, 0, ?, ?)
        "#,
    )
    .bind(&user_id)
    .bind(&request.email)
    .bind(&request.name)
    .bind(&password_hash)
    .bind(&role_str)
    .bind(can_create_invites)
    .bind(max_invites)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create user: {}", e);
        AppError::Database(e)
    })?;

    if result.rows_affected() != 1 {
        return Err(AppError::Internal {
            message: "Failed to create user".to_string(),
        });
    }

    // Return the created user
    get_user_by_id(pool, &user_id).await
}

async fn get_default_invite_limit(pool: &DatabasePool) -> Result<i32, AppError> {
    let limit = sqlx::query_scalar!(
        "SELECT value FROM admin_settings WHERE key = 'default_user_invite_limit'"
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(limit.and_then(|v| v.parse::<i32>().ok()).unwrap_or(5))
}

async fn get_max_total_users(pool: &DatabasePool) -> Result<i32, AppError> {
    let max_users =
        sqlx::query_scalar!("SELECT value FROM admin_settings WHERE key = 'max_total_users'")
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?;

    Ok(max_users
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1000))
}

pub async fn get_user_by_id(pool: &DatabasePool, user_id: &str) -> Result<User, AppError> {
    let user_row = sqlx::query_as::<_, UserRow>(
        "SELECT id, email, name, first_day_of_week, preferred_units, password_hash, role, can_create_invites, max_invites, invites_created, created_at, updated_at FROM users WHERE id = ?"
    )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch user by id: {}", e);
            AppError::Database(e)
        })?;

    user_row.map_or_else(
        || {
            Err(AppError::NotFound {
                resource: format!("User with id {user_id}"),
            })
        },
        UserRow::to_user,
    )
}

pub async fn get_user_by_email(pool: &DatabasePool, email: &str) -> Result<User, AppError> {
    let user_row = sqlx::query_as::<_, UserRow>(
        "SELECT id, email, name, first_day_of_week, preferred_units, password_hash, role, can_create_invites, max_invites, invites_created, created_at, updated_at FROM users WHERE email = ?"
    )
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch user by email: {}", e);
            AppError::Database(e)
        })?;

    user_row.map_or_else(
        || {
            Err(AppError::NotFound {
                resource: format!("User with email {email}"),
            })
        },
        UserRow::to_user,
    )
}

pub async fn verify_password(
    pool: &DatabasePool,
    email: &str,
    password: &str,
) -> Result<User, AppError> {
    let user = get_user_by_email(pool, email).await?;

    // Parse the stored password hash and verify with Argon2id
    let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|e| AppError::Internal {
        message: format!("Failed to parse password hash: {e}"),
    })?;

    let argon2 = get_argon2_config();
    let is_valid = argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();

    if is_valid {
        Ok(user)
    } else {
        Err(AppError::Authentication {
            message: "Invalid credentials".to_string(),
        })
    }
}

pub async fn update_user_login_time(pool: &DatabasePool, user_id: &str) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query!("UPDATE users SET updated_at = ? WHERE id = ?", now, user_id)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update user login time: {}", e);
            AppError::Database(e)
        })?;

    if result.rows_affected() != 1 {
        return Err(AppError::NotFound {
            resource: format!("User with id {user_id}"),
        });
    }

    Ok(())
}

pub async fn update_user_profile(
    pool: &DatabasePool,
    user_id: &str,
    name: &str,
    email: &str,
    first_day_of_week: &FirstDayOfWeek,
    preferred_units: &PreferredUnits,
) -> Result<User, AppError> {
    let existing =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = ? AND id != ?")
            .bind(email)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;

    if existing > 0 {
        return Err(AppError::Validation(duplicate_email_validation_error()));
    }

    let now = Utc::now().to_rfc3339();
    let updated = sqlx::query(
        "UPDATE users SET name = ?, email = ?, first_day_of_week = ?, preferred_units = ?, updated_at = ? WHERE id = ?",
    )
        .bind(name)
        .bind(email)
        .bind(first_day_of_week.to_string())
        .bind(preferred_units.to_string())
        .bind(now)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

    if updated.rows_affected() != 1 {
        return Err(AppError::NotFound {
            resource: format!("User with id {user_id}"),
        });
    }

    get_user_by_id(pool, user_id).await
}

pub async fn set_user_profile_picture(
    pool: &DatabasePool,
    user_id: &str,
    image_data: &[u8],
    content_type: &str,
) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();
    let updated = sqlx::query(
        "UPDATE users SET profile_picture_data = ?, profile_picture_content_type = ?, updated_at = ? WHERE id = ?",
    )
    .bind(image_data)
    .bind(content_type)
    .bind(now)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    if updated.rows_affected() != 1 {
        return Err(AppError::NotFound {
            resource: format!("User with id {user_id}"),
        });
    }

    Ok(())
}

pub async fn get_user_profile_picture(
    pool: &DatabasePool,
    user_id: &str,
) -> Result<Option<(Vec<u8>, String)>, AppError> {
    let row = sqlx::query(
        "SELECT profile_picture_data, profile_picture_content_type FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    let row = row.ok_or_else(|| AppError::NotFound {
        resource: format!("User with id {user_id}"),
    })?;

    let data = row
        .try_get::<Option<Vec<u8>>, _>("profile_picture_data")
        .map_err(AppError::Database)?;
    let content_type = row
        .try_get::<Option<String>, _>("profile_picture_content_type")
        .map_err(AppError::Database)?;

    Ok(match (data, content_type) {
        (Some(data), Some(content_type)) => Some((data, content_type)),
        _ => None,
    })
}

pub async fn delete_user_profile_picture(
    pool: &DatabasePool,
    user_id: &str,
) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();
    let updated = sqlx::query(
        "UPDATE users SET profile_picture_data = NULL, profile_picture_content_type = NULL, updated_at = ? WHERE id = ?",
    )
    .bind(now)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    if updated.rows_affected() != 1 {
        return Err(AppError::NotFound {
            resource: format!("User with id {user_id}"),
        });
    }

    Ok(())
}

pub async fn change_user_password(
    pool: &DatabasePool,
    user_id: &str,
    current_password: &str,
    new_password: &str,
) -> Result<(), AppError> {
    let user = get_user_by_id(pool, user_id).await?;
    verify_password(pool, &user.email, current_password).await?;

    let password_hash = hash_password(new_password)?;
    let now = Utc::now().to_rfc3339();

    let updated = sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?")
        .bind(password_hash)
        .bind(now)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

    if updated.rows_affected() != 1 {
        return Err(AppError::NotFound {
            resource: format!("User with id {user_id}"),
        });
    }

    Ok(())
}

pub async fn export_user_data(pool: &DatabasePool, user_id: &str) -> Result<Value, AppError> {
    let user = sqlx::query(
        "SELECT id, email, name, first_day_of_week, preferred_units, role, can_create_invites, max_invites, invites_created, created_at, updated_at FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound {
        resource: format!("User with id {user_id}"),
    })?;

    let plants = sqlx::query(
        "SELECT id, name, genus, created_at, updated_at FROM plants WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let tracking_entries = sqlx::query(
        "SELECT te.id, te.plant_id, te.timestamp, te.care_task_ids, te.measurements, te.notes, te.photo_ids, te.created_at, te.updated_at
         FROM tracking_entries te
         JOIN plants p ON p.id = te.plant_id
         WHERE p.user_id = ?
         ORDER BY te.timestamp DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let photos = sqlx::query(
        "SELECT ph.id, ph.plant_id, ph.filename, ph.original_filename, ph.size, ph.content_type, ph.width, ph.height, ph.created_at
         FROM photos ph
         JOIN plants p ON p.id = ph.plant_id
         WHERE p.user_id = ?
         ORDER BY ph.created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let invites = sqlx::query(
        "SELECT id, code, max_uses, current_uses, is_active, created_at, expires_at
         FROM invite_codes WHERE created_by = ? ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(json!({
        "exported_at": Utc::now().to_rfc3339(),
        "user": {
            "id": user.try_get::<String, _>("id").unwrap_or_default(),
            "email": user.try_get::<String, _>("email").unwrap_or_default(),
            "name": user.try_get::<String, _>("name").unwrap_or_default(),
            "role": user.try_get::<String, _>("role").unwrap_or_else(|_| "user".to_string()),
            "can_create_invites": user.try_get::<bool, _>("can_create_invites").unwrap_or(false),
            "max_invites": user.try_get::<Option<i32>, _>("max_invites").ok().flatten(),
            "invites_created": user.try_get::<i32, _>("invites_created").unwrap_or(0),
            "created_at": user.try_get::<String, _>("created_at").unwrap_or_default(),
            "updated_at": user.try_get::<String, _>("updated_at").unwrap_or_default(),
        },
        "plants": plants.into_iter().map(|row| json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "name": row.try_get::<String, _>("name").unwrap_or_default(),
            "genus": row.try_get::<String, _>("genus").unwrap_or_default(),
            "created_at": row.try_get::<String, _>("created_at").unwrap_or_default(),
            "updated_at": row.try_get::<String, _>("updated_at").unwrap_or_default(),
        })).collect::<Vec<_>>(),
        "tracking_entries": tracking_entries.into_iter().map(|row| json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "plant_id": row.try_get::<String, _>("plant_id").unwrap_or_default(),
            "timestamp": row.try_get::<String, _>("timestamp").unwrap_or_default(),
            "care_task_ids": row.try_get::<Option<String>, _>("care_task_ids").ok().flatten(),
            "measurements": row.try_get::<Option<String>, _>("measurements").ok().flatten(),
            "notes": row.try_get::<Option<String>, _>("notes").ok().flatten(),
            "photo_ids": row.try_get::<Option<String>, _>("photo_ids").ok().flatten(),
            "created_at": row.try_get::<String, _>("created_at").unwrap_or_default(),
            "updated_at": row.try_get::<String, _>("updated_at").unwrap_or_default(),
        })).collect::<Vec<_>>(),
        "photos": photos.into_iter().map(|row| json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "plant_id": row.try_get::<String, _>("plant_id").unwrap_or_default(),
            "filename": row.try_get::<String, _>("filename").unwrap_or_default(),
            "original_filename": row.try_get::<String, _>("original_filename").unwrap_or_default(),
            "size": row.try_get::<i64, _>("size").unwrap_or(0),
            "content_type": row.try_get::<String, _>("content_type").unwrap_or_default(),
            "width": row.try_get::<Option<i32>, _>("width").ok().flatten(),
            "height": row.try_get::<Option<i32>, _>("height").ok().flatten(),
            "created_at": row.try_get::<String, _>("created_at").unwrap_or_default(),
        })).collect::<Vec<_>>(),
        "invites": invites.into_iter().map(|row| json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "code": row.try_get::<String, _>("code").unwrap_or_default(),
            "max_uses": row.try_get::<i32, _>("max_uses").unwrap_or(1),
            "current_uses": row.try_get::<i32, _>("current_uses").unwrap_or(0),
            "is_active": row.try_get::<bool, _>("is_active").unwrap_or(false),
            "created_at": row.try_get::<String, _>("created_at").unwrap_or_default(),
            "expires_at": row.try_get::<Option<String>, _>("expires_at").ok().flatten(),
        })).collect::<Vec<_>>(),
    }))
}

pub async fn delete_user_account(
    pool: &DatabasePool,
    user_id: &str,
    current_password: &str,
) -> Result<(), AppError> {
    let user = get_user_by_id(pool, user_id).await?;
    verify_password(pool, &user.email, current_password).await?;

    let deleted = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

    if deleted.rows_affected() != 1 {
        return Err(AppError::NotFound {
            resource: format!("User with id {user_id}"),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::password_hash::{rand_core::OsRng, SaltString};

    #[test]
    fn test_password_hashing_and_verification() {
        let password = "test_password_123";

        // Test hashing with Argon2id
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = get_argon2_test_config();
        let hash_result = argon2.hash_password(password.as_bytes(), &salt);
        assert!(hash_result.is_ok());

        let password_hash = hash_result.unwrap().to_string();
        assert!(!password_hash.is_empty());
        assert_ne!(password_hash, password); // Hash should be different from original

        // Test verification with correct password
        let parsed_hash = PasswordHash::new(&password_hash).unwrap();
        let verify_result = argon2.verify_password(password.as_bytes(), &parsed_hash);
        assert!(verify_result.is_ok());

        // Test verification with incorrect password
        let wrong_password = "wrong_password";
        let verify_wrong_result = argon2.verify_password(wrong_password.as_bytes(), &parsed_hash);
        assert!(verify_wrong_result.is_err());
    }

    #[test]
    fn test_password_hash_uniqueness() {
        let password = "same_password";
        let argon2 = get_argon2_test_config();

        let salt1 = SaltString::generate(&mut OsRng);
        let salt2 = SaltString::generate(&mut OsRng);

        let hash1 = argon2
            .hash_password(password.as_bytes(), &salt1)
            .unwrap()
            .to_string();
        let hash2 = argon2
            .hash_password(password.as_bytes(), &salt2)
            .unwrap()
            .to_string();

        // Even with the same password, hashes should be different due to salt
        assert_ne!(hash1, hash2);

        // But both should verify correctly
        let parsed_hash1 = PasswordHash::new(&hash1).unwrap();
        let parsed_hash2 = PasswordHash::new(&hash2).unwrap();
        assert!(argon2
            .verify_password(password.as_bytes(), &parsed_hash1)
            .is_ok());
        assert!(argon2
            .verify_password(password.as_bytes(), &parsed_hash2)
            .is_ok());
    }

    #[test]
    fn test_empty_password_handling() {
        let empty_password = "";
        let argon2 = get_argon2_test_config();

        // Should be able to hash empty password (though not recommended)
        let salt = SaltString::generate(&mut OsRng);
        let hash_result = argon2.hash_password(empty_password.as_bytes(), &salt);
        assert!(hash_result.is_ok());

        let password_hash = hash_result.unwrap().to_string();
        let parsed_hash = PasswordHash::new(&password_hash).unwrap();
        assert!(argon2
            .verify_password(empty_password.as_bytes(), &parsed_hash)
            .is_ok());
        assert!(argon2
            .verify_password("not_empty".as_bytes(), &parsed_hash)
            .is_err());
    }

    #[test]
    fn test_long_password_handling() {
        let long_password = "a".repeat(100); // Long but reasonable password
        let argon2 = get_argon2_test_config();

        let salt = SaltString::generate(&mut OsRng);
        let hash_result = argon2.hash_password(long_password.as_bytes(), &salt);
        assert!(hash_result.is_ok());

        let password_hash = hash_result.unwrap().to_string();
        let parsed_hash = PasswordHash::new(&password_hash).unwrap();
        assert!(argon2
            .verify_password(long_password.as_bytes(), &parsed_hash)
            .is_ok());

        // Different password should not verify
        let different_password = "b".repeat(100);
        assert!(argon2
            .verify_password(different_password.as_bytes(), &parsed_hash)
            .is_err());
    }

    #[test]
    fn test_special_characters_in_password() {
        let special_password = "p@ssw0rd!#$%^&*()_+-=[]{}|;:'\",.<>?/~`";
        let argon2 = get_argon2_test_config();

        let salt = SaltString::generate(&mut OsRng);
        let hash_result = argon2.hash_password(special_password.as_bytes(), &salt);
        assert!(hash_result.is_ok());

        let password_hash = hash_result.unwrap().to_string();
        let parsed_hash = PasswordHash::new(&password_hash).unwrap();
        assert!(argon2
            .verify_password(special_password.as_bytes(), &parsed_hash)
            .is_ok());

        // Should not verify with different special characters
        let different_special = "p@ssw0rd!#$%^&*()_+-=[]{}|;:'\",.<>?/~";
        assert!(argon2
            .verify_password(different_special.as_bytes(), &parsed_hash)
            .is_err());
    }

    #[test]
    fn test_unicode_password_handling() {
        let unicode_password = "pásswörd123🔒";
        let argon2 = get_argon2_test_config();

        let salt = SaltString::generate(&mut OsRng);
        let hash_result = argon2.hash_password(unicode_password.as_bytes(), &salt);
        assert!(hash_result.is_ok());

        let password_hash = hash_result.unwrap().to_string();
        let parsed_hash = PasswordHash::new(&password_hash).unwrap();
        assert!(argon2
            .verify_password(unicode_password.as_bytes(), &parsed_hash)
            .is_ok());

        // Different unicode should not verify
        let different_unicode = "pásswörd123🔓"; // Different emoji
        assert!(argon2
            .verify_password(different_unicode.as_bytes(), &parsed_hash)
            .is_err());
    }

    #[test]
    fn test_case_sensitivity() {
        let password = "TestPassword123";
        let argon2 = get_argon2_test_config();

        let salt = SaltString::generate(&mut OsRng);
        let hash_result = argon2.hash_password(password.as_bytes(), &salt);
        assert!(hash_result.is_ok());

        let password_hash = hash_result.unwrap().to_string();
        let parsed_hash = PasswordHash::new(&password_hash).unwrap();
        assert!(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok());

        // Different case should not verify
        assert!(argon2
            .verify_password("testpassword123".as_bytes(), &parsed_hash)
            .is_err());
        assert!(argon2
            .verify_password("TESTPASSWORD123".as_bytes(), &parsed_hash)
            .is_err());
        assert!(argon2
            .verify_password("TestPASSWORD123".as_bytes(), &parsed_hash)
            .is_err());
    }

    #[test]
    fn test_invalid_hash_format() {
        let password = "test_password";
        let invalid_hash = "not_a_valid_argon2_hash";
        let argon2 = get_argon2_test_config();

        // Should handle invalid hash gracefully
        let parse_result = PasswordHash::new(invalid_hash);
        assert!(parse_result.is_err());

        // If we somehow get a parsed but invalid hash, verification should fail
        if let Ok(parsed_hash) = parse_result {
            let verify_result = argon2.verify_password(password.as_bytes(), &parsed_hash);
            assert!(verify_result.is_err());
        }
    }

    #[tokio::test]
    async fn test_max_total_users_applies_to_admin_users() {
        let pool = crate::database::create_pool_with_url("sqlite::memory:")
            .await
            .expect("Failed to create db pool");
        crate::database::run_migrations(&pool)
            .await
            .expect("Failed to run migrations");

        sqlx::query!("UPDATE admin_settings SET value = '1' WHERE key = 'max_total_users'")
            .execute(&pool)
            .await
            .expect("Failed to update max_total_users");

        let first_admin = CreateUserRequest {
            name: "Admin One".to_string(),
            email: "admin1@test.com".to_string(),
            password: "password123".to_string(),
            invite_code: None,
        };
        create_user_internal(&pool, &first_admin, UserRole::Admin, true, Some(50))
            .await
            .expect("First admin should be created");

        let second_admin = CreateUserRequest {
            name: "Admin Two".to_string(),
            email: "admin2@test.com".to_string(),
            password: "password123".to_string(),
            invite_code: None,
        };
        let result =
            create_user_internal(&pool, &second_admin, UserRole::Admin, true, Some(50)).await;

        match result {
            Err(AppError::Internal { message }) => {
                assert_eq!(message, "Maximum number of users reached");
            }
            other => panic!("Expected max user limit error, got: {other:?}"),
        }
    }
}
