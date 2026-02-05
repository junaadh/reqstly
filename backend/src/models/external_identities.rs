use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{CreateUser, User},
};

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq,
)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AuthProvider {
    AzureAd,
    Passkey,
    Password,
}

impl From<String> for AuthProvider {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "azure_ad" => Self::AzureAd,
            "passkey" => Self::Passkey,
            "password" => Self::Password,
            _ => panic!("Invalid auth provider: {value}"),
        }
    }
}

impl From<&str> for AuthProvider {
    fn from(value: &str) -> Self {
        Self::from(value.to_lowercase())
    }
}

impl std::fmt::Display for AuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AzureAd => write!(f, "azure_ad"),
            Self::Passkey => write!(f, "passkey"),
            Self::Password => write!(f, "password"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExternalIdentity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: AuthProvider,
    pub subject: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl ExternalIdentity {
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        provider: AuthProvider,
        subject: &str,
        email: Option<&str>,
    ) -> Result<Self, AppError> {
        sqlx::query_as!(
            ExternalIdentity,
            r#"
            INSERT INTO external_identities (user_id, provider, subject, email)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, provider as "provider: _", subject, email, created_at
            "#,
            user_id,
            provider.to_string(),
            subject,
            email
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn find_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<ExternalIdentity>, AppError> {
        sqlx::query_as!(
            ExternalIdentity,
            r#"
            SELECT id, user_id, provider as "provider: _", subject, email, created_at
            FROM external_identities
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn find_by_provider_subject(
        pool: &PgPool,
        provider: AuthProvider,
        subject: &str,
    ) -> Result<Option<Self>, AppError> {
        sqlx::query_as!(
            ExternalIdentity,
            r#"
            SELECT id, user_id, provider as "provider: _",
                   subject, email, created_at
            FROM external_identities
            WHERE provider = $1 AND subject = $2
            "#,
            provider.to_string(),
            subject
        )
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn resolve_user_from_external_identity(
        pool: &PgPool,
        provider: AuthProvider,
        subject: &str,
        email: Option<&str>,
        name: Option<&str>,
    ) -> Result<User, AppError> {
        // 1. Identity already exists → load user
        if let Some(identity) =
            ExternalIdentity::find_by_provider_subject(pool, provider, subject)
                .await?
        {
            return User::find_by_id(pool, identity.user_id).await?.ok_or(
                AppError::Internal(
                    "External Identity found but cascaded User not found"
                        .to_string(),
                ),
            );
        }

        // 2. Try linking to existing user by email
        let user = if let Some(email) = email {
            if let Some(user) = User::find_by_email(pool, email).await? {
                user
            } else {
                User::create(
                    pool,
                    CreateUser {
                        email: email.to_string(),
                        name: name.unwrap_or(email).to_string(),
                    },
                )
                .await?
            }
        } else {
            return Err(AppError::Unauthorized(
                "Azure identity missing email".into(),
            ));
        };

        // 3. Create external identity link
        ExternalIdentity::create(pool, user.id, provider, subject, email)
            .await?;

        Ok(user)
    }
}
