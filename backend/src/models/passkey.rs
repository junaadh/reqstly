use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Passkey credential stored for WebAuthn authentication
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PasskeyCredential {
    pub id: Uuid,
    pub user_id: Uuid,
    pub credential_id: String,
    pub public_key: String,
    pub counter: i32,
    pub transports: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a new passkey credential
#[derive(Debug, Deserialize)]
pub struct CreatePasskeyCredential {
    pub user_id: Uuid,
    pub credential_id: String,
    pub public_key: String,
    pub transports: Option<Vec<String>>,
}

impl PasskeyCredential {
    /// Create a new passkey credential
    pub async fn create(
        pool: &PgPool,
        credential: CreatePasskeyCredential,
    ) -> Result<PasskeyCredential, AppError> {
        let id = Uuid::new_v4();

        sqlx::query_as!(
            PasskeyCredential,
            r#"
            INSERT INTO passkey_credentials (id, user_id, credential_id, public_key, transports)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, credential_id, public_key, counter, transports, created_at
            "#,
            id,
            credential.user_id,
            credential.credential_id,
            credential.public_key,
            credential.transports.as_deref()
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    /// Find a passkey credential by credential ID
    pub async fn find_by_credential_id(
        pool: &PgPool,
        credential_id: &str,
    ) -> Result<Option<PasskeyCredential>, AppError> {
        sqlx::query_as!(
            PasskeyCredential,
            r#"
            SELECT id, user_id, credential_id, public_key, counter, transports, created_at
            FROM passkey_credentials
            WHERE credential_id = $1
            "#,
            credential_id
        )
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)
    }

    /// Find all passkey credentials for a user
    pub async fn find_by_user_id(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<PasskeyCredential>, AppError> {
        sqlx::query_as!(
            PasskeyCredential,
            r#"
            SELECT id, user_id, credential_id, public_key, counter, transports, created_at
            FROM passkey_credentials
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    /// Update the signature counter for a credential
    /// This is used for replay protection in WebAuthn
    pub async fn update_counter(
        pool: &PgPool,
        credential_id: &str,
        counter: u32,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE passkey_credentials
            SET counter = $2
            WHERE credential_id = $1
            "#,
            credential_id,
            counter as i32
        )
        .execute(pool)
        .await
        .map_err(AppError::from)?;

        Ok(())
    }

    /// Delete a passkey credential
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        sqlx::query!("DELETE FROM passkey_credentials WHERE id = $1", id)
            .execute(pool)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    /// Count the number of passkeys for a user
    pub async fn count_by_user_id(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<i64, AppError> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM passkey_credentials
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

        Ok(result.count.unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_create_passkey_credential() {
        let pool = setup_test_pool().await;

        let user_id = Uuid::new_v4();
        let credential = PasskeyCredential::create(
            &pool,
            CreatePasskeyCredential {
                user_id,
                credential_id: "test-credential-id".to_string(),
                public_key: "test-public-key".to_string(),
                transports: Some(vec!["internal".to_string()]),
            },
        )
        .await
        .unwrap();

        assert_eq!(credential.user_id, user_id);
        assert_eq!(credential.credential_id, "test-credential-id");
        assert_eq!(credential.counter, 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_find_by_credential_id() {
        let pool = setup_test_pool().await;

        let user_id = Uuid::new_v4();
        let credential = PasskeyCredential::create(
            &pool,
            CreatePasskeyCredential {
                user_id,
                credential_id: "find-me-credential".to_string(),
                public_key: "test-public-key".to_string(),
                transports: None,
            },
        )
        .await
        .unwrap();

        let found = PasskeyCredential::find_by_credential_id(
            &pool,
            "find-me-credential",
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(found.id, credential.id);
        assert_eq!(found.user_id, user_id);
    }

    #[tokio::test]
    #[ignore]
    async fn test_update_counter() {
        let pool = setup_test_pool().await;

        let user_id = Uuid::new_v4();
        let _credential = PasskeyCredential::create(
            &pool,
            CreatePasskeyCredential {
                user_id,
                credential_id: "counter-credential".to_string(),
                public_key: "test-public-key".to_string(),
                transports: None,
            },
        )
        .await
        .unwrap();

        // Update counter to 5
        PasskeyCredential::update_counter(&pool, "counter-credential", 5)
            .await
            .unwrap();

        let updated = PasskeyCredential::find_by_credential_id(
            &pool,
            "counter-credential",
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(updated.counter, 5);
    }

    async fn setup_test_pool() -> PgPool {
        panic!("Test database not configured");
    }
}
