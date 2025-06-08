use axum::{
    async_trait,
    extract::{FromRequest, Request},
    response::Response,
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::utils::errors::{AppError, Result};

pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}