use crate::error::AppError;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Password model for password-based authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Password {
    pub id: Uuid,
    pub user_id: Uuid,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a new password
#[derive(Debug, Deserialize)]
pub struct CreatePassword {
    pub user_id: Uuid,
    pub password: String,
}

/// Input for password login
#[derive(Debug, Deserialize)]
pub struct PasswordLogin {
    pub email: String,
    pub password: String,
}

/// Input for password signup
#[derive(Debug, Deserialize)]
pub struct PasswordSignup {
    pub email: String,
    pub name: String,
    pub password: String,
}

impl Password {
    /// Create a new password for a user (hashes the password)
    pub async fn create(
        pool: &PgPool,
        input: CreatePassword,
    ) -> Result<Password, AppError> {
        let password_hash = hash(&input.password, DEFAULT_COST)
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;

        let id = Uuid::new_v4();

        let row = sqlx::query(
            r#"
            INSERT INTO passwords (id, user_id, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, password_hash, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(input.user_id)
        .bind(&password_hash)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

        Ok(Password {
            id: row.get("id"),
            user_id: row.get("user_id"),
            password_hash: row.get("password_hash"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Verify a password against the stored hash
    pub fn verify(&self, password: &str) -> Result<bool, AppError> {
        verify(password, &self.password_hash)
            .map_err(|e| AppError::Internal(format!("Failed to verify password: {}", e)))
    }

    /// Find password by user ID
    pub async fn find_by_user_id(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Option<Password>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, password_hash, created_at, updated_at
            FROM passwords
            WHERE user_id = $1
            "#
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)?;

        match row {
            Some(r) => Ok(Some(Password {
                id: r.get("id"),
                user_id: r.get("user_id"),
                password_hash: r.get("password_hash"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })),
            None => Ok(None),
        }
    }

    /// Update password for a user
    pub async fn update(
        pool: &PgPool,
        user_id: Uuid,
        new_password: &str,
    ) -> Result<Password, AppError> {
        let password_hash = hash(new_password, DEFAULT_COST)
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;

        let row = sqlx::query(
            r#"
            UPDATE passwords
            SET password_hash = $2, updated_at = NOW()
            WHERE user_id = $1
            RETURNING id, user_id, password_hash, created_at, updated_at
            "#
        )
        .bind(user_id)
        .bind(&password_hash)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

        Ok(Password {
            id: row.get("id"),
            user_id: row.get("user_id"),
            password_hash: row.get("password_hash"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Delete password for a user
    pub async fn delete(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM passwords WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_verification() {
        let password = "test_password_123";
        let hash_result = hash(password, DEFAULT_COST).unwrap();
        assert!(verify(password, &hash_result).unwrap());
        assert!(!verify("wrong_password", &hash_result).unwrap());
    }
}
