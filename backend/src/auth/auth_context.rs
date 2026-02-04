use axum::async_trait;

use crate::{
    auth::session_token::SessionToken,
    AppState,
    error::AppError,
    models::{
        Session, User,
        external_identities::{AuthProvider, ExternalIdentity},
    },
};

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub session: Session,
    pub user: User,
    pub identity: Option<ExternalIdentity>,
}

impl AuthContext {
    pub fn provider(&self) -> AuthProvider {
        self.session.provider
    }

    pub fn is_federated(&self) -> bool {
        self.identity.is_some()
    }

    pub fn require_provider(
        &self,
        provider: AuthProvider,
    ) -> Result<(), AppError> {
        if self.session.provider == provider {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "Invalid auth provider: {}",
                provider
            )))
        }
    }
}

#[async_trait]
impl axum::extract::FromRequestParts<AppState> for AuthContext {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = SessionToken::from_request_parts(parts, state).await?;

        let (session, identity) =
            Session::find_valid(&state.db, &token).await?.ok_or_else(|| {
                AppError::Unauthorized("Invalid or expired session".into())
            })?;

        if session.is_expired() {
            return Err(AppError::Unauthorized("Session expired".into()));
        }

        let user = User::find_by_id(&state.db, session.user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

        Ok(Self {
            session,
            user,
            identity,
        })
    }
}
