use crate::utils::errors::Result;
use sqlx::SqlitePool;
use tracing::info;

pub async fn ensure_admin_invite(pool: &SqlitePool) -> Result<String> {
    // Check if any admin users exist
    let admin_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE role = 'admin'"
    )
    .fetch_one(pool)
    .await?;

    if admin_count > 0 {
        info!("Admin user already exists, skipping admin invite creation");
        return Ok("Admin user already exists".to_string());
    }

    // Check if an admin invite already exists
    let admin_invite_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM invite_codes WHERE id LIKE 'admin-%' AND is_active = TRUE"
    )
    .fetch_one(pool)
    .await?;

    if admin_invite_count > 0 {
        info!("Admin invite already exists, skipping creation");
        return Ok("Admin invite already exists".to_string());
    }

    // Create a special admin invite code
    let admin_invite_code = generate_admin_invite_code();
    
    // Insert the admin invite directly with custom code
    let invite_id = uuid::Uuid::new_v4().to_string();
    let admin_invite_id = format!("admin-{}", invite_id);
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query!(
        r#"
        INSERT INTO invite_codes (id, code, created_by, max_uses, current_uses, is_active, created_at, updated_at)
        VALUES (?, ?, NULL, 1, 0, TRUE, ?, ?)
        "#,
        admin_invite_id,
        admin_invite_code,
        now,
        now
    )
    .execute(pool)
    .await?;

    let invite_url = format!("http://localhost:3000/invite?code={}", admin_invite_code);

    // Print the admin invite to console
    println!("\n🚀 PLANTY ADMIN SETUP");
    println!("====================");
    println!("🎫 Admin Invite Code: {}", admin_invite_code);
    println!("🔗 Admin Signup Link: {}", invite_url);
    println!("====================");
    println!("⚠️  IMPORTANT: Use this code to create the first admin account!");
    println!("⚠️  This code can only be used ONCE to create an admin user.");
    println!("⚠️  Save this code securely - it will not be shown again.\n");

    Ok(format!("Admin invite created: {}", admin_invite_code))
}

fn generate_admin_invite_code() -> String {
    format!("ADMIN-{}", generate_secure_code(8))
}

fn generate_secure_code(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub async fn get_system_stats(pool: &SqlitePool) -> Result<SystemStats> {
    let total_users = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    let total_invites = sqlx::query_scalar!("SELECT COUNT(*) FROM invite_codes")
        .fetch_one(pool)
        .await?;

    let active_invites = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM invite_codes WHERE is_active = TRUE AND (expires_at IS NULL OR expires_at > datetime('now'))"
    )
    .fetch_one(pool)
    .await?;

    let used_invites = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM invite_codes WHERE current_uses > 0"
    )
    .fetch_one(pool)
    .await?;

    let admin_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE role = 'admin'"
    )
    .fetch_one(pool)
    .await?;

    let max_users_setting = sqlx::query_scalar!(
        "SELECT value FROM admin_settings WHERE key = 'max_total_users'"
    )
    .fetch_optional(pool)
    .await?;

    let max_total_users = max_users_setting
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1000);

    Ok(SystemStats {
        total_users,
        max_total_users,
        total_invites,
        active_invites,
        used_invites,
        admin_count,
    })
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct SystemStats {
    pub total_users: i32,
    pub max_total_users: i32,
    pub total_invites: i32,
    pub active_invites: i32,
    pub used_invites: i32,
    pub admin_count: i32,
}