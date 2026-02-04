use axum::{async_trait, http::request::Parts};
use tower_cookies::Cookies;

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionToken(String);

impl SessionToken {
    pub fn new(token: String) -> Self {
        Self(token)
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for SessionToken {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

#[async_trait]
impl<S> axum::extract::FromRequestParts<S> for SessionToken
where
    S: Send + Sync,
{
    type Rejection = AppError;

    #[doc = " Perform the extraction."]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn from_request_parts<'life0, 'life1>(
        parts: &'life0 mut Parts,
        state: &'life1 S,
    ) -> Result<Self, Self::Rejection>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
    {
        let cookies = Cookies::from_request_parts(parts, state).await.map_err(
            |(_, err)| {
                AppError::Unauthorized(format!("Invalid Cookies: {err}"))
            },
        )?;

        let cookie = cookies.get("session").ok_or_else(|| {
            AppError::Unauthorized("Missing session cookie".to_owned())
        })?;

        Ok(SessionToken(cookie.value().to_owned()))
    }
}
