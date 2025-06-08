use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;
use chrono::Utc;
use anyhow::Result;

use crate::database::DatabasePool;
use crate::models::{CreateUserRequest, User, UserRow};
use crate::utils::errors::AppError;

pub async fn create_user(
    pool: &DatabasePool,
    request: &CreateUserRequest,
) -> Result<User, AppError> {
    // Check if user with this email already exists
    if get_user_by_email(pool, &request.email).await.is_ok() {
        return Err(AppError::Validation(
            validator::ValidationErrors::new() // TODO: Add proper validation error
        ));
    }

    let user_id = Uuid::new_v4().to_string();
    let salt = Uuid::new_v4().to_string();
    let password_hash = hash(&request.password, DEFAULT_COST)
        .map_err(|e| AppError::Internal {
            message: format!("Failed to hash password: {e}"),
        })?;
    
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query!(
        r#"
        INSERT INTO users (id, email, name, password_hash, salt, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        user_id,
        request.email,
        request.name,
        password_hash,
        salt,
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

pub async fn get_user_by_id(pool: &DatabasePool, user_id: &str) -> Result<User, AppError> {
    let user_row = sqlx::query_as::<_, UserRow>(
        "SELECT * FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch user by id: {}", e);
        AppError::Database(e)
    })?;

    user_row.map_or_else(|| Err(AppError::NotFound {
            resource: format!("User with id {user_id}"),
        }), UserRow::to_user)
}

pub async fn get_user_by_email(pool: &DatabasePool, email: &str) -> Result<User, AppError> {
    let user_row = sqlx::query_as::<_, UserRow>(
        "SELECT * FROM users WHERE email = ?"
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch user by email: {}", e);
        AppError::Database(e)
    })?;

    user_row.map_or_else(|| Err(AppError::NotFound {
            resource: format!("User with email {email}"),
        }), UserRow::to_user)
}

pub async fn verify_password(
    pool: &DatabasePool,
    email: &str,
    password: &str,
) -> Result<User, AppError> {
    let user = get_user_by_email(pool, email).await?;
    
    let is_valid = verify(password, &user.password_hash)
        .map_err(|e| AppError::Internal {
            message: format!("Failed to verify password: {e}"),
        })?;

    if is_valid {
        Ok(user)
    } else {
        Err(AppError::Authentication {
            message: "Invalid credentials".to_string(),
        })
    }
}

pub async fn update_user_login_time(
    pool: &DatabasePool,
    user_id: &str,
) -> Result<(), AppError> {
    let now = Utc::now().to_rfc3339();
    
    let result = sqlx::query!(
        "UPDATE users SET updated_at = ? WHERE id = ?",
        now,
        user_id
    )
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
    use bcrypt::{hash, verify, DEFAULT_COST};

    #[test]
    fn test_password_hashing_and_verification() {
        let password = "test_password_123";
        
        // Test hashing
        let hash_result = hash(password, DEFAULT_COST);
        assert!(hash_result.is_ok());
        
        let password_hash = hash_result.unwrap();
        assert!(!password_hash.is_empty());
        assert_ne!(password_hash, password); // Hash should be different from original
        
        // Test verification with correct password
        let verify_result = verify(password, &password_hash);
        assert!(verify_result.is_ok());
        assert!(verify_result.unwrap());
        
        // Test verification with incorrect password
        let wrong_password = "wrong_password";
        let verify_wrong_result = verify(wrong_password, &password_hash);
        assert!(verify_wrong_result.is_ok());
        assert!(!verify_wrong_result.unwrap());
    }

    #[test]
    fn test_password_hash_uniqueness() {
        let password = "same_password";
        
        let hash1 = hash(password, DEFAULT_COST).unwrap();
        let hash2 = hash(password, DEFAULT_COST).unwrap();
        
        // Even with the same password, hashes should be different due to salt
        assert_ne!(hash1, hash2);
        
        // But both should verify correctly
        assert!(verify(password, &hash1).unwrap());
        assert!(verify(password, &hash2).unwrap());
    }

    #[test]
    fn test_empty_password_handling() {
        let empty_password = "";
        
        // Should be able to hash empty password (though not recommended)
        let hash_result = hash(empty_password, DEFAULT_COST);
        assert!(hash_result.is_ok());
        
        let password_hash = hash_result.unwrap();
        assert!(verify(empty_password, &password_hash).unwrap());
        assert!(!verify("not_empty", &password_hash).unwrap());
    }

    #[test]
    fn test_long_password_handling() {
        let long_password = "a".repeat(100); // Long but reasonable password
        
        let hash_result = hash(&long_password, DEFAULT_COST);
        assert!(hash_result.is_ok());
        
        let password_hash = hash_result.unwrap();
        assert!(verify(&long_password, &password_hash).unwrap());
        
        // Different password should not verify
        let different_password = "b".repeat(100);
        assert!(!verify(&different_password, &password_hash).unwrap());
    }

    #[test]
    fn test_special_characters_in_password() {
        let special_password = "p@ssw0rd!#$%^&*()_+-=[]{}|;:'\",.<>?/~`";
        
        let hash_result = hash(special_password, DEFAULT_COST);
        assert!(hash_result.is_ok());
        
        let password_hash = hash_result.unwrap();
        assert!(verify(special_password, &password_hash).unwrap());
        
        // Should not verify with different special characters
        let different_special = "p@ssw0rd!#$%^&*()_+-=[]{}|;:'\",.<>?/~";
        assert!(!verify(different_special, &password_hash).unwrap());
    }

    #[test]
    fn test_unicode_password_handling() {
        let unicode_password = "pásswörd123🔒";
        
        let hash_result = hash(unicode_password, DEFAULT_COST);
        assert!(hash_result.is_ok());
        
        let password_hash = hash_result.unwrap();
        assert!(verify(unicode_password, &password_hash).unwrap());
        
        // Different unicode should not verify
        let different_unicode = "pásswörd123🔓"; // Different emoji
        assert!(!verify(different_unicode, &password_hash).unwrap());
    }

    #[test]
    fn test_case_sensitivity() {
        let password = "TestPassword123";
        
        let hash_result = hash(password, DEFAULT_COST);
        assert!(hash_result.is_ok());
        
        let password_hash = hash_result.unwrap();
        assert!(verify(password, &password_hash).unwrap());
        
        // Different case should not verify
        assert!(!verify("testpassword123", &password_hash).unwrap());
        assert!(!verify("TESTPASSWORD123", &password_hash).unwrap());
        assert!(!verify("TestPASSWORD123", &password_hash).unwrap());
    }

    #[test]
    fn test_invalid_hash_format() {
        let password = "test_password";
        let invalid_hash = "not_a_valid_bcrypt_hash";
        
        // Should handle invalid hash gracefully
        let verify_result = verify(password, invalid_hash);
        assert!(verify_result.is_err());
    }
}