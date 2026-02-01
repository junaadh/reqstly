use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

/// User model representing a user in the system
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub azure_ad_subject: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a new user
#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub name: String,
    #[serde(default)]
    pub azure_ad_subject: Option<String>,
}

impl User {
    /// Find a user by ID
    pub async fn find_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<User>, AppError> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id, email, name, azure_ad_subject, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)
    }

    /// Find a user by email
    pub async fn find_by_email(
        pool: &PgPool,
        email: &str,
    ) -> Result<Option<User>, AppError> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id, email, name, azure_ad_subject, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)
    }

    /// Find a user by Azure AD subject identifier
    pub async fn find_by_azure_subject(
        pool: &PgPool,
        subject: &str,
    ) -> Result<Option<User>, AppError> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id, email, name, azure_ad_subject, created_at, updated_at
            FROM users
            WHERE azure_ad_subject = $1
            "#,
            subject
        )
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)
    }

    /// Create a new user
    pub async fn create(
        pool: &PgPool,
        user: CreateUser,
    ) -> Result<User, AppError> {
        let id = Uuid::new_v4();

        sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, email, name, azure_ad_subject)
            VALUES ($1, $2, $3, $4)
            RETURNING id, email, name, azure_ad_subject, created_at, updated_at
            "#,
            id,
            user.email,
            user.name,
            user.azure_ad_subject
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    /// Create or update a user from Azure AD
    /// Returns the user (either newly created or updated)
    pub async fn create_from_azure(
        pool: &PgPool,
        subject: &str,
        email: &str,
        name: &str,
    ) -> Result<User, AppError> {
        // First, try to find by Azure AD subject
        if let Some(mut user) =
            User::find_by_azure_subject(pool, subject).await?
        {
            // Update email and name in case they changed
            user.email = email.to_string();
            user.name = name.to_string();

            sqlx::query_as!(
                User,
                r#"
                UPDATE users
                SET email = $2, name = $3
                WHERE id = $1
                RETURNING id, email, name, azure_ad_subject, created_at, updated_at
                "#,
                user.id,
                user.email,
                user.name
            )
            .fetch_one(pool)
            .await
            .map_err(AppError::from)
        } else {
            // Check if user with this email already exists
            if let Some(mut user) = User::find_by_email(pool, email).await? {
                // Link to Azure AD
                user.azure_ad_subject = Some(subject.to_string());

                sqlx::query_as!(
                    User,
                    r#"
                    UPDATE users
                    SET azure_ad_subject = $2
                    WHERE id = $1
                    RETURNING id, email, name, azure_ad_subject, created_at, updated_at
                    "#,
                    user.id,
                    user.azure_ad_subject
                )
                .fetch_one(pool)
                .await
                .map_err(AppError::from)
            } else {
                // Create new user
                User::create(
                    pool,
                    CreateUser {
                        email: email.to_string(),
                        name: name.to_string(),
                        azure_ad_subject: Some(subject.to_string()),
                    },
                )
                .await
            }
        }
    }

    /// Update user information
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        user: CreateUser,
    ) -> Result<User, AppError> {
        sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET email = $2, name = $3, azure_ad_subject = $4
            WHERE id = $1
            RETURNING id, email, name, azure_ad_subject, created_at, updated_at
            "#,
            id,
            user.email,
            user.name,
            user.azure_ad_subject
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    /// Delete a user
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        sqlx::query!("DELETE FROM users WHERE id = $1", id)
            .execute(pool)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    /// List all users
    pub async fn list(pool: &PgPool) -> Result<Vec<User>, AppError> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id, email, name, azure_ad_subject, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_create_user() {
        let pool = setup_test_pool().await;

        let user = User::create(
            &pool,
            CreateUser {
                email: "test@example.com".to_string(),
                name: "Test User".to_string(),
                azure_ad_subject: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.name, "Test User");
        assert!(user.azure_ad_subject.is_none());
        // assert!(user.id.version() != 0); // Valid UUID
    }

    #[tokio::test]
    #[ignore]
    async fn test_find_user_by_email() {
        let pool = setup_test_pool().await;

        let user = User::create(
            &pool,
            CreateUser {
                email: "findme@example.com".to_string(),
                name: "Find Me".to_string(),
                azure_ad_subject: None,
            },
        )
        .await
        .unwrap();

        let found = User::find_by_email(&pool, "findme@example.com")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(found.id, user.id);
        assert_eq!(found.email, user.email);
    }

    #[tokio::test]
    #[ignore]
    async fn test_find_by_azure_subject() {
        let pool = setup_test_pool().await;

        let user = User::create(
            &pool,
            CreateUser {
                email: "azure@example.com".to_string(),
                name: "Azure User".to_string(),
                azure_ad_subject: Some("azure-subject-123".to_string()),
            },
        )
        .await
        .unwrap();

        let found = User::find_by_azure_subject(&pool, "azure-subject-123")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(found.id, user.id);
        assert_eq!(
            found.azure_ad_subject,
            Some("azure-subject-123".to_string())
        );
    }

    async fn setup_test_pool() -> PgPool {
        // In a real test, you'd set up a test database
        // For now, this is a placeholder
        panic!("Test database not configured");
    }
}
