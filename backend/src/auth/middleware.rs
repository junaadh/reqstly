use axum::async_trait;

use crate::{
    AppState, auth::auth_context::AuthContext, error::AppError,
    models::external_identities::AuthProvider,
};

#[derive(Debug, Clone)]
pub struct OptionalAuth(pub Option<AuthContext>);

impl OptionalAuth {}

#[async_trait]
impl axum::extract::FromRequestParts<AppState> for OptionalAuth {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match AuthContext::from_request_parts(parts, state).await {
            Ok(ctx) => Ok(OptionalAuth(Some(ctx))),
            Err(_) => Ok(OptionalAuth(None)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AzureUser(pub AuthContext);

#[async_trait]
impl axum::extract::FromRequestParts<AppState> for AzureUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let ctx = AuthContext::from_request_parts(parts, state).await?;

        ctx.require_provider(AuthProvider::AzureAd)?;

        Ok(Self(ctx))
    }
}

#[derive(Debug, Clone)]
pub struct PasskeyUser(pub AuthContext);

#[async_trait]
impl axum::extract::FromRequestParts<AppState> for PasskeyUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let ctx = AuthContext::from_request_parts(parts, state).await?;

        ctx.require_provider(AuthProvider::Passkey)?;

        Ok(Self(ctx))
    }
}
