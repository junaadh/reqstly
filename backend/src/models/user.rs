use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// User model representing a user in the system
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a new user
#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub name: String,
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
            SELECT id, email, name, created_at, updated_at
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
            SELECT id, email, name, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
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
        sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (email, name)
            VALUES ($1, $2)
            RETURNING id, email, name, created_at, updated_at
            "#,
            user.email,
            user.name
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
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
            SET email = $2, name = $3
            WHERE id = $1
            RETURNING id, email, name, created_at, updated_at
            "#,
            id,
            user.email,
            user.name,
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
            SELECT id, email, name, created_at, updated_at
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
            },
        )
        .await
        .unwrap();

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.name, "Test User");
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

    async fn setup_test_pool() -> PgPool {
        // In a real test, you'd set up a test database
        // For now, this is a placeholder
        panic!("Test database not configured");
    }
}
