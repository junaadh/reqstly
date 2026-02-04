use crate::auth::session_token::SessionToken;
use crate::models::external_identities::ExternalIdentity;
use crate::{error::AppError, models::external_identities::AuthProvider};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

/// Session model for authenticated users
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub external_identity_id: Option<Uuid>,
    pub provider: AuthProvider,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Default session expiration time (24 hours)
const DEFAULT_SESSION_DURATION_HOURS: i64 = 24;

impl Session {
    /// Create a new session for a user
    /// Generates a secure random token and stores its hash
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        identity: Option<ExternalIdentity>,
        provider: AuthProvider,
    ) -> Result<(Session, SessionToken), AppError> {
        let id = Uuid::new_v4();

        // Generate secure random token (32 bytes = 256 bits)
        let token = generate_session_token();

        // Hash the token using SHA-256 before storing
        let token_hash = hash_token(token.as_ref());

        // Set expiration time
        let expires_at =
            Utc::now() + Duration::hours(DEFAULT_SESSION_DURATION_HOURS);

        let external_identity_id = identity.as_ref().map(|i| i.id);

        sqlx::query_as!(
            Session,
            r#"
            INSERT INTO sessions (id, user_id, external_identity_id, provider, token_hash, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, external_identity_id, provider as "provider: _", token_hash, expires_at, created_at
            "#,
            id,
            user_id,
            external_identity_id,
            provider.to_string(),
            token_hash,
            expires_at
        )
        .fetch_one(pool)
        .await
        .map(|session| (session, token))
        .map_err(AppError::from)
    }

    /// Find a valid session by token
    /// Returns None if session doesn't exist, is expired, or token doesn't match
    pub async fn find_valid(
        pool: &PgPool,
        token: &SessionToken,
    ) -> Result<Option<(Session, Option<ExternalIdentity>)>, AppError> {
        // Hash the provided token
        let token_hash = hash_token(token.as_ref());

        // Find session by token hash
        let session: Session = match sqlx::query_as!(
            Session,
            r#"
            SELECT id, user_id, external_identity_id, provider as "provider: _", token_hash, expires_at, created_at
            FROM sessions
            WHERE token_hash = $1 AND expires_at > NOW()
            "#,
            token_hash
        )
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)? {
            Some(s) => s,
            None => return Ok(None)
        };

        let external_identity =
            if let Some(ext_id) = session.external_identity_id {
                ExternalIdentity::find_by_id(pool, ext_id).await?
            } else {
                None
            };

        Ok(Some((session, external_identity)))
    }

    /// Invalidate a session by token
    pub async fn invalidate(
        pool: &PgPool,
        token: &SessionToken,
    ) -> Result<(), AppError> {
        let token_hash = hash_token(token.as_ref());

        sqlx::query!("DELETE FROM sessions WHERE token_hash = $1", token_hash)
            .execute(pool)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    /// Invalidate all sessions for a user
    pub async fn invalidate_all_for_user(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!("DELETE FROM sessions WHERE user_id = $1", user_id)
            .execute(pool)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    /// Clean up expired sessions
    /// Returns the number of sessions deleted
    pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, AppError> {
        let result =
            sqlx::query!("DELETE FROM sessions WHERE expires_at <= NOW()")
                .execute(pool)
                .await
                .map_err(AppError::from)?;

        Ok(result.rows_affected())
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }

    /// Extend session expiration time
    pub async fn extend(&self, pool: &PgPool) -> Result<Session, AppError> {
        let new_expires_at =
            Utc::now() + Duration::hours(DEFAULT_SESSION_DURATION_HOURS);

        sqlx::query_as!(
            Session,
            r#"
            UPDATE sessions
            SET expires_at = $2
            WHERE id = $1
            RETURNING id, user_id, external_identity_id, provider as "provider: _", token_hash, expires_at, created_at
            "#,
            self.id,
            new_expires_at
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }
}

/// Generate a secure random session token
fn generate_session_token() -> SessionToken {
    use rand::Rng;
    const TOKEN_SIZE: usize = 32; // 256 bits

    let mut bytes = [0u8; TOKEN_SIZE];
    rand::thread_rng().fill(&mut bytes);

    // Encode as base64url (URL-safe without padding)
    SessionToken::new(base64_url_encode(&bytes))
}

/// Hash a session token using SHA-256
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    base64_url_encode(&result)
}

/// Encode bytes as URL-safe base64 without padding
fn base64_url_encode(data: &[u8]) -> String {
    use base64::prelude::*;

    let encoded = BASE64_URL_SAFE_NO_PAD.encode(data);
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_create_session() {
        let pool = setup_test_pool().await;

        let user_id = Uuid::new_v4();
        let (session, token) =
            Session::create(&pool, user_id, None, AuthProvider::AzureAd)
                .await
                .unwrap();

        assert_eq!(session.user_id, user_id);
        assert!(!token.as_ref().is_empty());
        assert!(!session.token_hash.is_empty());
        assert!(session.token_hash != token.as_ref()); // Hash should be different from token
        assert!(!session.is_expired());
    }

    #[tokio::test]
    #[ignore]
    async fn test_find_valid_session() {
        let pool = setup_test_pool().await;

        let user_id = Uuid::new_v4();
        let (session, token) =
            Session::create(&pool, user_id, None, AuthProvider::Passkey)
                .await
                .unwrap();

        let found = Session::find_valid(&pool, &token).await.unwrap().unwrap();

        assert_eq!(found.0.id, session.id);
        assert_eq!(found.0.user_id, user_id);
    }

    #[tokio::test]
    #[ignore]
    async fn test_invalidate_session() {
        let pool = setup_test_pool().await;

        let user_id = Uuid::new_v4();
        let (_session, token) =
            Session::create(&pool, user_id, None, AuthProvider::AzureAd)
                .await
                .unwrap();

        // Invalidate the session
        Session::invalidate(&pool, &token).await.unwrap();

        // Should not be found
        let found = Session::find_valid(&pool, &token).await.unwrap();
        assert!(found.is_none());
    }

    async fn setup_test_pool() -> PgPool {
        panic!("Test database not configured");
    }
}
