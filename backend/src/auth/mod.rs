use axum_login::{
    tower_sessions::{Expiry, MemoryStore, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use time::Duration;

use crate::database::{users as db_users, DatabasePool};
use crate::models::User;
use crate::utils::errors::AppError;

// Define our authentication backend
#[derive(Clone, Debug)]
pub struct AuthBackend {
    pub db: DatabasePool,
}

impl AuthBackend {
    #[must_use]
    pub const fn new(db: DatabasePool) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl axum_login::AuthnBackend for AuthBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = AppError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        match db_users::verify_password(&self.db, &creds.email, &creds.password).await {
            Ok(user) => {
                // Update login time
                let _ = db_users::update_user_login_time(&self.db, &user.id).await;
                Ok(Some(user))
            }
            Err(AppError::Authentication { .. } | AppError::NotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn get_user(&self, user_id: &String) -> Result<Option<Self::User>, Self::Error> {
        match db_users::get_user_by_id(&self.db, user_id).await {
            Ok(user) => Ok(Some(user)),
            Err(AppError::NotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[async_trait::async_trait]
impl axum_login::AuthzBackend for AuthBackend {
    type Permission = ();

    async fn get_group_permissions(
        &self,
        _user: &Self::User,
    ) -> Result<std::collections::HashSet<Self::Permission>, Self::Error> {
        Ok(std::collections::HashSet::new())
    }
}

#[derive(Clone, Debug)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

// Type aliases for convenience
pub type AuthSession = axum_login::AuthSession<AuthBackend>;

// Helper function to create session and auth layers
#[must_use]
pub fn create_auth_layers(
    pool: DatabasePool,
) -> (SessionManagerLayer<MemoryStore>, axum_login::AuthManagerLayer<AuthBackend, MemoryStore>) {
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS
        .with_expiry(Expiry::OnInactivity(Duration::seconds(3600))); // 1 hour

    let backend = AuthBackend::new(pool);
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer.clone()).build();

    (session_layer, auth_layer)
}