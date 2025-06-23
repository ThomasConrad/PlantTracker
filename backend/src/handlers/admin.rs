use axum::{
    extract::{Query, State},
    response::Json,
    routing::{delete, get, post, put},
    Json as JsonExtractor, Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    admin::{get_system_stats, SystemStats},
    app_state::AppState,
    auth::AuthSession,
    models::user::{UserResponse, UserRole},
    utils::errors::{AppError, Result},
};

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AdminDashboardResponse {
    pub system_stats: SystemStats,
    pub recent_users: Vec<UserResponse>,
    pub recent_invites: Vec<InviteInfo>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct InviteInfo {
    pub id: String,
    pub code: String,
    pub created_by: Option<String>,
    pub created_by_name: Option<String>,
    pub max_uses: i32,
    pub current_uses: i32,
    pub is_active: bool,
    pub expires_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserListResponse {
    pub users: Vec<UserResponse>,
    pub total: i32,
    pub page: i32,
    pub limit: i32,
    pub total_pages: i32,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateUserRequest {
    pub role: Option<UserRole>,
    pub can_create_invites: Option<bool>,
    pub max_invites: Option<Option<i32>>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AdminSettingsResponse {
    pub max_total_users: i32,
    pub default_user_invite_limit: i32,
    pub registration_enabled: bool,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateAdminSettingsRequest {
    pub max_total_users: Option<i32>,
    pub default_user_invite_limit: Option<i32>,
    pub registration_enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BulkUserActionRequest {
    pub user_ids: Vec<String>,
    pub action: BulkUserAction,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BulkUserAction {
    Delete,
    SetRole(UserRole),
    EnableInvites,
    DisableInvites,
}

/// Get admin dashboard data
#[utoipa::path(
    get,
    path = "/admin/dashboard",
    responses(
        (status = 200, description = "Admin dashboard data", body = AdminDashboardResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required")
    ),
    security(("session" = []))
)]
pub async fn get_admin_dashboard(
    auth_session: AuthSession,
    State(state): State<AppState>,
) -> Result<Json<AdminDashboardResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Authentication required".to_string(),
    })?;

    // Check if user is admin
    if !user.is_admin() {
        return Err(AppError::Authorization {
            message: "Admin access required".to_string(),
        });
    }

    let system_stats = get_system_stats(&state.pool).await?;

    // Get recent users (last 10)
    let recent_users_rows = sqlx::query!(
        r#"
        SELECT id, email, name, role, can_create_invites, max_invites, invites_created, created_at, updated_at
        FROM users 
        ORDER BY created_at DESC 
        LIMIT 10
        "#
    )
    .fetch_all(&state.pool)
    .await?;

    let recent_users: Vec<UserResponse> = recent_users_rows
        .into_iter()
        .map(|row| {
            Ok(UserResponse {
                id: row.id,
                email: row.email,
                name: row.name,
                role: row.role.parse().unwrap_or(UserRole::User),
                can_create_invites: row.can_create_invites,
                max_invites: row.max_invites.map(|v| v as i32),
                invites_created: row.invites_created as i32,
                invites_remaining: row
                    .max_invites
                    .map(|max| (max as i32) - (row.invites_created as i32)),
                created_at: row
                    .created_at
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .unwrap_or_default(),
                updated_at: row
                    .updated_at
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .unwrap_or_default(),
            })
        })
        .filter_map(Result::ok)
        .collect();

    // Get recent invites (last 10)
    let recent_invites_rows = sqlx::query!(
        r#"
        SELECT 
            ic.id, ic.code, ic.created_by, ic.max_uses, ic.current_uses, 
            ic.is_active, ic.expires_at, ic.created_at,
            u.name as created_by_name
        FROM invite_codes ic
        LEFT JOIN users u ON ic.created_by = u.id
        ORDER BY ic.created_at DESC 
        LIMIT 10
        "#
    )
    .fetch_all(&state.pool)
    .await?;

    let recent_invites: Vec<InviteInfo> = recent_invites_rows
        .into_iter()
        .map(|row| InviteInfo {
            id: row.id.unwrap_or_default(),
            code: row.code,
            created_by: row.created_by,
            created_by_name: row.created_by_name,
            max_uses: row.max_uses as i32,
            current_uses: row.current_uses as i32,
            is_active: row.is_active,
            expires_at: row.expires_at,
            created_at: row.created_at,
        })
        .collect();

    Ok(Json(AdminDashboardResponse {
        system_stats,
        recent_users,
        recent_invites,
    }))
}

/// List all users with pagination
#[utoipa::path(
    get,
    path = "/admin/users",
    params(
        ("page" = Option<i32>, Query, description = "Page number (default: 1)"),
        ("limit" = Option<i32>, Query, description = "Items per page (default: 20)"),
        ("role" = Option<String>, Query, description = "Filter by role")
    ),
    responses(
        (status = 200, description = "List of users", body = UserListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required")
    ),
    security(("session" = []))
)]
pub async fn list_users(
    auth_session: AuthSession,
    State(state): State<AppState>,
    Query(query): Query<UserListQuery>,
) -> Result<Json<UserListResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Authentication required".to_string(),
    })?;

    // Check if user is admin
    if !user.is_admin() {
        return Err(AppError::Authorization {
            message: "Admin access required".to_string(),
        });
    }

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;

    // Removed complex parameter handling - using direct query approach instead

    // Use a single query with CASE to handle optional role filtering
    let role_condition = query.role.as_deref().unwrap_or("%");
    let use_role_filter_int = if query.role.is_some() { 1i32 } else { 0i32 };

    let users_rows = sqlx::query!(
        r#"
        SELECT id, email, name, role, can_create_invites, max_invites, invites_created, created_at, updated_at
        FROM users 
        WHERE (? = 0 OR role = ?)
        ORDER BY created_at DESC 
        LIMIT ? OFFSET ?
        "#,
        use_role_filter_int,
        role_condition,
        limit,
        offset
    )
    .fetch_all(&state.pool)
    .await?;

    let users: Vec<UserResponse> = users_rows
        .into_iter()
        .map(|row| UserResponse {
            id: row.id,
            email: row.email,
            name: row.name,
            role: row.role.parse().unwrap_or(UserRole::User),
            can_create_invites: row.can_create_invites,
            max_invites: row.max_invites.map(|v| v as i32),
            invites_created: row.invites_created as i32,
            invites_remaining: row
                .max_invites
                .map(|max| (max as i32) - (row.invites_created as i32)),
            created_at: row.created_at.parse().unwrap_or_default(),
            updated_at: row.updated_at.parse().unwrap_or_default(),
        })
        .collect();

    let total = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE (? = 0 OR role = ?)",
        use_role_filter_int,
        role_condition
    )
    .fetch_one(&state.pool)
    .await?;

    let total_pages = (total as f64 / limit as f64).ceil() as i32;

    Ok(Json(UserListResponse {
        users,
        total,
        page,
        limit,
        total_pages,
    }))
}

/// Update a user's role and permissions
#[utoipa::path(
    put,
    path = "/admin/users/{user_id}",
    params(
        ("user_id" = String, Path, description = "User ID to update")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = UserResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "User not found")
    ),
    security(("session" = []))
)]
pub async fn update_user(
    auth_session: AuthSession,
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
    JsonExtractor(request): JsonExtractor<UpdateUserRequest>,
) -> Result<Json<UserResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Authentication required".to_string(),
    })?;

    // Check if user is admin
    if !user.is_admin() {
        return Err(AppError::Authorization {
            message: "Admin access required".to_string(),
        });
    }

    // Prevent users from modifying themselves
    if user.id == user_id {
        return Err(AppError::Authorization {
            message: "Cannot modify your own account".to_string(),
        });
    }

    // Check if target user exists
    sqlx::query!(
        "SELECT id, email, name, role, can_create_invites, max_invites, invites_created, created_at, updated_at FROM users WHERE id = ?",
        user_id
    )
    .fetch_optional(&state.pool)
    .await?.ok_or_else(|| AppError::NotFound {
        resource: "User not found".to_string(),
    })?;

    // Validate that at least one field is being updated
    if request.role.is_none()
        && request.can_create_invites.is_none()
        && request.max_invites.is_none()
    {
        return Err(AppError::Authorization {
            message: "No updates provided".to_string(),
        });
    }

    // Execute individual updates - simpler approach for SQLite
    let now = chrono::Utc::now().to_rfc3339();

    if let Some(role) = &request.role {
        let role_str = role.to_string();
        sqlx::query!(
            "UPDATE users SET role = ?, updated_at = ? WHERE id = ?",
            role_str,
            now,
            user_id
        )
        .execute(&state.pool)
        .await?;
    }

    if let Some(can_create_invites) = request.can_create_invites {
        sqlx::query!(
            "UPDATE users SET can_create_invites = ?, updated_at = ? WHERE id = ?",
            can_create_invites,
            now,
            user_id
        )
        .execute(&state.pool)
        .await?;
    }

    if let Some(max_invites) = request.max_invites {
        sqlx::query!(
            "UPDATE users SET max_invites = ?, updated_at = ? WHERE id = ?",
            max_invites,
            now,
            user_id
        )
        .execute(&state.pool)
        .await?;
    }

    // Fetch updated user
    let updated_user = sqlx::query!(
        "SELECT id, email, name, role, can_create_invites, max_invites, invites_created, created_at, updated_at FROM users WHERE id = ?",
        user_id
    )
    .fetch_one(&state.pool)
    .await?;

    let user_response = UserResponse {
        id: updated_user.id,
        email: updated_user.email,
        name: updated_user.name,
        role: updated_user.role.parse().unwrap_or(UserRole::User),
        can_create_invites: updated_user.can_create_invites,
        max_invites: updated_user.max_invites.map(|v| v as i32),
        invites_created: updated_user.invites_created as i32,
        invites_remaining: updated_user
            .max_invites
            .map(|max| (max as i32) - (updated_user.invites_created as i32)),
        created_at: updated_user.created_at.parse().unwrap_or_default(),
        updated_at: updated_user.updated_at.parse().unwrap_or_default(),
    };

    Ok(Json(user_response))
}

/// Delete a user (admin only)
#[utoipa::path(
    delete,
    path = "/admin/users/{user_id}",
    params(
        ("user_id" = String, Path, description = "User ID to delete")
    ),
    responses(
        (status = 200, description = "User deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "User not found")
    ),
    security(("session" = []))
)]
pub async fn delete_user(
    auth_session: AuthSession,
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Authentication required".to_string(),
    })?;

    // Check if user is admin
    if !user.is_admin() {
        return Err(AppError::Authorization {
            message: "Admin access required".to_string(),
        });
    }

    // Prevent users from deleting themselves
    if user.id == user_id {
        return Err(AppError::Authorization {
            message: "Cannot delete your own account".to_string(),
        });
    }

    // Check if target user exists
    let target_user = sqlx::query!("SELECT id FROM users WHERE id = ?", user_id)
        .fetch_optional(&state.pool)
        .await?;

    if target_user.is_none() {
        return Err(AppError::NotFound {
            resource: "User not found".to_string(),
        });
    }

    // Delete user (cascading deletes should handle related data)
    sqlx::query!("DELETE FROM users WHERE id = ?", user_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "User deleted successfully"
    })))
}

/// Get admin settings
#[utoipa::path(
    get,
    path = "/admin/settings",
    responses(
        (status = 200, description = "Admin settings", body = AdminSettingsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required")
    ),
    security(("session" = []))
)]
pub async fn get_admin_settings(
    auth_session: AuthSession,
    State(state): State<AppState>,
) -> Result<Json<AdminSettingsResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Authentication required".to_string(),
    })?;

    // Check if user is admin
    if !user.is_admin() {
        return Err(AppError::Authorization {
            message: "Admin access required".to_string(),
        });
    }

    let max_total_users_opt =
        sqlx::query_scalar!("SELECT value FROM admin_settings WHERE key = 'max_total_users'")
            .fetch_one(&state.pool)
            .await?;

    let max_total_users = max_total_users_opt.parse::<i32>().unwrap_or(1000);

    let default_user_invite_limit_opt = sqlx::query_scalar!(
        "SELECT value FROM admin_settings WHERE key = 'default_user_invite_limit'"
    )
    .fetch_one(&state.pool)
    .await?;

    let default_user_invite_limit = default_user_invite_limit_opt.parse::<i32>().unwrap_or(5);

    let registration_enabled_opt =
        sqlx::query_scalar!("SELECT value FROM admin_settings WHERE key = 'registration_enabled'")
            .fetch_one(&state.pool)
            .await?;

    let registration_enabled = registration_enabled_opt.parse::<bool>().unwrap_or(true);

    Ok(Json(AdminSettingsResponse {
        max_total_users,
        default_user_invite_limit,
        registration_enabled,
    }))
}

/// Update admin settings
#[utoipa::path(
    put,
    path = "/admin/settings",
    request_body = UpdateAdminSettingsRequest,
    responses(
        (status = 200, description = "Settings updated successfully", body = AdminSettingsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required")
    ),
    security(("session" = []))
)]
pub async fn update_admin_settings(
    auth_session: AuthSession,
    State(state): State<AppState>,
    JsonExtractor(request): JsonExtractor<UpdateAdminSettingsRequest>,
) -> Result<Json<AdminSettingsResponse>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Authentication required".to_string(),
    })?;

    // Check if user is admin
    if !user.is_admin() {
        return Err(AppError::Authorization {
            message: "Admin access required".to_string(),
        });
    }

    let now = chrono::Utc::now().to_rfc3339();

    if let Some(max_total_users) = request.max_total_users {
        let value_str = max_total_users.to_string();
        sqlx::query!(
            "UPDATE admin_settings SET value = ?, updated_at = ? WHERE key = 'max_total_users'",
            value_str,
            now
        )
        .execute(&state.pool)
        .await?;
    }

    if let Some(default_user_invite_limit) = request.default_user_invite_limit {
        let value_str = default_user_invite_limit.to_string();
        sqlx::query!(
            "UPDATE admin_settings SET value = ?, updated_at = ? WHERE key = 'default_user_invite_limit'",
            value_str,
            now
        )
        .execute(&state.pool)
        .await?;
    }

    if let Some(registration_enabled) = request.registration_enabled {
        let value_str = registration_enabled.to_string();
        sqlx::query!(
            "UPDATE admin_settings SET value = ?, updated_at = ? WHERE key = 'registration_enabled'",
            value_str,
            now
        )
        .execute(&state.pool)
        .await?;
    }

    // Return updated settings by fetching them again
    let max_total_users_opt =
        sqlx::query_scalar!("SELECT value FROM admin_settings WHERE key = 'max_total_users'")
            .fetch_one(&state.pool)
            .await?;

    let max_total_users = max_total_users_opt.parse::<i32>().unwrap_or(1000);

    let default_user_invite_limit_opt = sqlx::query_scalar!(
        "SELECT value FROM admin_settings WHERE key = 'default_user_invite_limit'"
    )
    .fetch_one(&state.pool)
    .await?;

    let default_user_invite_limit = default_user_invite_limit_opt.parse::<i32>().unwrap_or(5);

    let registration_enabled_opt =
        sqlx::query_scalar!("SELECT value FROM admin_settings WHERE key = 'registration_enabled'")
            .fetch_one(&state.pool)
            .await?;

    let registration_enabled = registration_enabled_opt.parse::<bool>().unwrap_or(true);

    Ok(Json(AdminSettingsResponse {
        max_total_users,
        default_user_invite_limit,
        registration_enabled,
    }))
}

/// Perform bulk actions on users
#[utoipa::path(
    post,
    path = "/admin/users/bulk",
    request_body = BulkUserActionRequest,
    responses(
        (status = 200, description = "Bulk action completed successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 400, description = "Invalid request")
    ),
    security(("session" = []))
)]
pub async fn bulk_user_action(
    auth_session: AuthSession,
    State(state): State<AppState>,
    JsonExtractor(request): JsonExtractor<BulkUserActionRequest>,
) -> Result<Json<serde_json::Value>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Authentication required".to_string(),
    })?;

    // Check if user is admin
    if !user.is_admin() {
        return Err(AppError::Authorization {
            message: "Admin access required".to_string(),
        });
    }

    if request.user_ids.is_empty() {
        return Err(AppError::Authorization {
            message: "No users specified".to_string(),
        });
    }

    // Prevent admin from performing bulk actions on themselves
    if request.user_ids.contains(&user.id) {
        return Err(AppError::Authorization {
            message: "Cannot perform bulk actions on your own account".to_string(),
        });
    }

    let now = chrono::Utc::now().to_rfc3339();
    let mut affected_count = 0;
    let action_debug = format!("{:?}", request.action);

    match request.action {
        BulkUserAction::Delete => {
            for user_id in &request.user_ids {
                let result = sqlx::query!("DELETE FROM users WHERE id = ?", user_id)
                    .execute(&state.pool)
                    .await?;
                affected_count += result.rows_affected();
            }
        }
        BulkUserAction::SetRole(role) => {
            let role_str = role.to_string();
            for user_id in &request.user_ids {
                let result = sqlx::query!(
                    "UPDATE users SET role = ?, updated_at = ? WHERE id = ?",
                    role_str,
                    now,
                    user_id
                )
                .execute(&state.pool)
                .await?;
                affected_count += result.rows_affected();
            }
        }
        BulkUserAction::EnableInvites => {
            for user_id in &request.user_ids {
                let result = sqlx::query!(
                    "UPDATE users SET can_create_invites = TRUE, updated_at = ? WHERE id = ?",
                    now,
                    user_id
                )
                .execute(&state.pool)
                .await?;
                affected_count += result.rows_affected();
            }
        }
        BulkUserAction::DisableInvites => {
            for user_id in &request.user_ids {
                let result = sqlx::query!(
                    "UPDATE users SET can_create_invites = FALSE, updated_at = ? WHERE id = ?",
                    now,
                    user_id
                )
                .execute(&state.pool)
                .await?;
                affected_count += result.rows_affected();
            }
        }
    }

    Ok(Json(serde_json::json!({
        "message": "Bulk action completed successfully",
        "affected_count": affected_count,
        "action": action_debug
    })))
}

/// Get system health information
#[utoipa::path(
    get,
    path = "/admin/health",
    responses(
        (status = 200, description = "System health information"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required")
    ),
    security(("session" = []))
)]
pub async fn get_system_health(
    auth_session: AuthSession,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let user = auth_session.user.ok_or(AppError::Authentication {
        message: "Authentication required".to_string(),
    })?;

    // Check if user is admin
    if !user.is_admin() {
        return Err(AppError::Authorization {
            message: "Admin access required".to_string(),
        });
    }

    // Check database connectivity
    let db_status = match sqlx::query_scalar!("SELECT 1").fetch_one(&state.pool).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    // Get database size information (SQLite specific)
    let db_page_count = sqlx::query_scalar!("PRAGMA page_count")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(Some(0))
        .unwrap_or(0) as i64;

    let db_page_size = sqlx::query_scalar!("PRAGMA page_size")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(Some(0))
        .unwrap_or(0) as i64;

    let db_size_bytes = db_page_count * db_page_size;

    // Get recent activity counts
    let users_last_24h = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE created_at > datetime('now', '-1 day')"
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let invites_last_24h = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM invite_codes WHERE created_at > datetime('now', '-1 day')"
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    Ok(Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "database": {
            "status": db_status,
            "size_bytes": db_size_bytes,
            "page_count": db_page_count,
            "page_size": db_page_size
        },
        "activity_24h": {
            "new_users": users_last_24h,
            "new_invites": invites_last_24h
        },
        "uptime": {
            "note": "Application uptime tracking not implemented"
        }
    })))
}

/// Admin routes  
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(get_admin_dashboard))
        .route("/users", get(list_users))
        .route("/users/:user_id", put(update_user))
        .route("/users/:user_id", delete(delete_user))
        .route("/users/bulk", post(bulk_user_action))
        .route(
            "/settings",
            get(get_admin_settings).put(update_admin_settings),
        )
        .route("/health", get(get_system_health))
}
