use anyhow::Result;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use chrono::Utc;
use uuid::Uuid;

// Argon2id configuration following OWASP recommendations:
// - 19 MiB of memory (19456 KB)
// - 2 iterations
// - 1 degree of parallelism
fn get_argon2_config() -> Argon2<'static> {
    Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(19456, 2, 1, None).unwrap()
    )
}

// For tests, use faster but still secure parameters
#[cfg(test)]
fn get_argon2_test_config() -> Argon2<'static> {
    Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(4096, 1, 1, None).unwrap() // Faster for tests
    )
}

use crate::database::DatabasePool;
use crate::models::{CreateUserRequest, User, UserRow, UserRole};
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
        return Err(AppError::Validation(
            validator::ValidationErrors::new(), // TODO: Add proper validation error
        ));
    }

    // Check total user limit
    let total_users = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

    let max_total_users = get_max_total_users(pool).await?;
    
    if total_users >= max_total_users && role != UserRole::Admin {
        return Err(AppError::Internal {
            message: "Maximum number of users reached".to_string(),
        });
    }

    let user_id = Uuid::new_v4().to_string();
    
    // Generate salt and hash password with Argon2id
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = get_argon2_config();
    let password_hash = argon2.hash_password(request.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal {
            message: format!("Failed to hash password: {e}"),
        })?
        .to_string();

    let now = Utc::now().to_rfc3339();
    let role_str = role.to_string();

    let result = sqlx::query!(
        r#"
        INSERT INTO users (id, email, name, password_hash, role, can_create_invites, max_invites, invites_created, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?)
        "#,
        user_id,
        request.email,
        request.name,
        password_hash,
        role_str,
        can_create_invites,
        max_invites,
        now,
        now
    )
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

    Ok(limit
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(5))
}

async fn get_max_total_users(pool: &DatabasePool) -> Result<i32, AppError> {
    let max_users = sqlx::query_scalar!(
        "SELECT value FROM admin_settings WHERE key = 'max_total_users'"
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(max_users
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1000))
}

pub async fn get_user_by_id(pool: &DatabasePool, user_id: &str) -> Result<User, AppError> {
    let user_row = sqlx::query_as::<_, UserRow>(
        "SELECT id, email, name, password_hash, role, can_create_invites, max_invites, invites_created, created_at, updated_at FROM users WHERE id = ?"
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
        "SELECT id, email, name, password_hash, role, can_create_invites, max_invites, invites_created, created_at, updated_at FROM users WHERE email = ?"
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
    let is_valid = argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok();

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
        
        let hash1 = argon2.hash_password(password.as_bytes(), &salt1).unwrap().to_string();
        let hash2 = argon2.hash_password(password.as_bytes(), &salt2).unwrap().to_string();

        // Even with the same password, hashes should be different due to salt
        assert_ne!(hash1, hash2);

        // But both should verify correctly
        let parsed_hash1 = PasswordHash::new(&hash1).unwrap();
        let parsed_hash2 = PasswordHash::new(&hash2).unwrap();
        assert!(argon2.verify_password(password.as_bytes(), &parsed_hash1).is_ok());
        assert!(argon2.verify_password(password.as_bytes(), &parsed_hash2).is_ok());
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
        assert!(argon2.verify_password(empty_password.as_bytes(), &parsed_hash).is_ok());
        assert!(argon2.verify_password("not_empty".as_bytes(), &parsed_hash).is_err());
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
        assert!(argon2.verify_password(long_password.as_bytes(), &parsed_hash).is_ok());

        // Different password should not verify
        let different_password = "b".repeat(100);
        assert!(argon2.verify_password(different_password.as_bytes(), &parsed_hash).is_err());
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
        assert!(argon2.verify_password(special_password.as_bytes(), &parsed_hash).is_ok());

        // Should not verify with different special characters
        let different_special = "p@ssw0rd!#$%^&*()_+-=[]{}|;:'\",.<>?/~";
        assert!(argon2.verify_password(different_special.as_bytes(), &parsed_hash).is_err());
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
        assert!(argon2.verify_password(unicode_password.as_bytes(), &parsed_hash).is_ok());

        // Different unicode should not verify
        let different_unicode = "pásswörd123🔓"; // Different emoji
        assert!(argon2.verify_password(different_unicode.as_bytes(), &parsed_hash).is_err());
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
        assert!(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok());

        // Different case should not verify
        assert!(argon2.verify_password("testpassword123".as_bytes(), &parsed_hash).is_err());
        assert!(argon2.verify_password("TESTPASSWORD123".as_bytes(), &parsed_hash).is_err());
        assert!(argon2.verify_password("TestPASSWORD123".as_bytes(), &parsed_hash).is_err());
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
}
