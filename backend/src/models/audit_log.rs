use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "varchar")]
#[serde(rename_all = "snake_case")] // frontend uses 'status_changed'
pub enum AuditAction {
    Created,
    Updated,
    Deleted,
    StatusChanged,
}

impl From<String> for AuditAction {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&str> for AuditAction {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "ceated" => Self::Created,
            "updated" => Self::Updated,
            "deleted" => Self::Deleted,
            "status_changed" => Self::StatusChanged,
            _ => panic!("Invalid audit action: {}", s),
        }
    }
}

impl From<AuditAction> for serde_json::Value {
    fn from(value: AuditAction) -> Self {
        Self::String(value.to_string())
    }
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Updated => write!(f, "updated"),
            Self::Deleted => write!(f, "deleted"),
            Self::StatusChanged => write!(f, "status_changed"),
        }
    }
}

/// Audit log entry for tracking changes to requests
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub request_id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: AuditAction,
    pub new_value: Option<serde_json::Value>,
    pub old_value: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl AuditLog {
    /// Create a new audit log entry
    pub async fn create(
        pool: &PgPool,
        request_id: Uuid,
        user_id: Uuid,
        action: AuditAction,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
    ) -> Result<AuditLog, AppError> {
        let id = Uuid::new_v4();

        sqlx::query_as!(
            AuditLog,
            r#"
            INSERT INTO audit_logs (id, request_id, user_id, action, old_value, new_value)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, request_id, user_id, action, old_value, new_value, created_at
            "#,
            id,
            request_id,
            user_id,
            action.to_string(),
            old_value,
            new_value
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    /// Find all audit logs for a specific request
    pub async fn find_by_request_id(
        pool: &PgPool,
        request_id: Uuid,
    ) -> Result<Vec<AuditLog>, AppError> {
        sqlx::query_as!(
            AuditLog,
            r#"
            SELECT id, request_id, user_id, action, old_value, new_value, created_at
            FROM audit_logs
            WHERE request_id = $1
            ORDER BY created_at DESC
            "#,
            request_id
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    /// Find all audit logs made by a specific user
    pub async fn find_by_changed_by(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<AuditLog>, AppError> {
        sqlx::query_as!(
            AuditLog,
            r#"
            SELECT id, request_id, user_id, action, old_value, new_value, created_at
            FROM audit_logs
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    /// Find audit logs for a specific request and field
    pub async fn find_by_request_and_field(
        pool: &PgPool,
        request_id: Uuid,
        action: AuditAction,
    ) -> Result<Vec<AuditLog>, AppError> {
        sqlx::query_as!(
            AuditLog,
            r#"
            SELECT id, request_id, user_id, action, old_value, new_value, created_at
            FROM audit_logs
            WHERE request_id = $1 AND action = $2
            ORDER BY created_at DESC
            "#,
            request_id,
            action.to_string()
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    /// Delete audit logs for a request (cascade delete should handle this)
    /// This is typically called when a request is deleted
    pub async fn delete_for_request(
        pool: &PgPool,
        request_id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "DELETE FROM audit_logs WHERE request_id = $1",
            request_id
        )
        .execute(pool)
        .await
        .map_err(AppError::from)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_create_audit_log() {
        let pool = setup_test_pool().await;

        let request_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let log = AuditLog::create(
            &pool,
            request_id,
            user_id,
            AuditAction::StatusChanged,
            "open".into(),
            "in_progress".into(),
        )
        .await
        .unwrap();

        assert_eq!(log.request_id, request_id);
        assert_eq!(log.user_id, Some(user_id));
        assert_eq!(log.action, AuditAction::StatusChanged);
        assert_eq!(log.old_value, Some("open".into()));
        assert_eq!(log.new_value, Some("in_progress".into()));
    }

    #[tokio::test]
    #[ignore]
    async fn test_find_by_request_id() {
        let pool = setup_test_pool().await;

        let request_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Create multiple audit logs for the same request
        AuditLog::create(
            &pool,
            request_id,
            user_id,
            AuditAction::StatusChanged,
            "open".into(),
            "in_progress".into(),
        )
        .await
        .unwrap();
        AuditLog::create(
            &pool,
            request_id,
            user_id,
            AuditAction::Updated,
            "low".into(),
            "high".into(),
        )
        .await
        .unwrap();

        let logs = AuditLog::find_by_request_id(&pool, request_id)
            .await
            .unwrap();

        assert_eq!(logs.len(), 2);
    }

    async fn setup_test_pool() -> PgPool {
        panic!("Test database not configured");
    }
}
