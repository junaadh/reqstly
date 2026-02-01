use crate::models::AuditLog;
use crate::{error::AppError, models::audit_log::AuditAction};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

/// Request status enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "varchar")]
pub enum RequestStatus {
    Open,
    InProgress,
    Resolved,
}

impl FromStr for RequestStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(RequestStatus::Open),
            "in_progress" => Ok(RequestStatus::InProgress),
            "resolved" => Ok(RequestStatus::Resolved),
            _ => Err(format!("Invalid request status: {}", s)),
        }
    }
}

impl From<RequestStatus> for serde_json::Value {
    fn from(value: RequestStatus) -> Self {
        serde_json::Value::String(value.to_string())
    }
}

impl std::fmt::Display for RequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestStatus::Open => write!(f, "open"),
            RequestStatus::InProgress => write!(f, "in_progress"),
            RequestStatus::Resolved => write!(f, "resolved"),
        }
    }
}

/// Request category enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "varchar")]
pub enum RequestCategory {
    IT,
    Ops,
    Admin,
    HR,
}

impl FromStr for RequestCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "IT" => Ok(RequestCategory::IT),
            "OPS" => Ok(RequestCategory::Ops),
            "ADMIN" => Ok(RequestCategory::Admin),
            "HR" => Ok(RequestCategory::HR),
            _ => Err(format!("Invalid request category: {}", s)),
        }
    }
}

impl std::fmt::Display for RequestCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestCategory::IT => write!(f, "IT"),
            RequestCategory::Ops => write!(f, "Ops"),
            RequestCategory::Admin => write!(f, "Admin"),
            RequestCategory::HR => write!(f, "HR"),
        }
    }
}

/// Request priority enum
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq,
)]
#[sqlx(type_name = "varchar")]
pub enum RequestPriority {
    Low,
    Medium,
    High,
}

impl FromStr for RequestPriority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(RequestPriority::Low),
            "medium" => Ok(RequestPriority::Medium),
            "high" => Ok(RequestPriority::High),
            _ => Err(format!("Invalid request priority: {}", s)),
        }
    }
}

impl From<RequestPriority> for serde_json::Value {
    fn from(value: RequestPriority) -> Self {
        Self::String(value.to_string())
    }
}

impl std::fmt::Display for RequestPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestPriority::Low => write!(f, "low"),
            RequestPriority::Medium => write!(f, "medium"),
            RequestPriority::High => write!(f, "high"),
        }
    }
}

/// Request model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Request {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub category: RequestCategory,
    pub status: RequestStatus,
    pub priority: RequestPriority,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a new request
#[derive(Debug, Deserialize)]
pub struct CreateRequest {
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub category: RequestCategory,
    pub priority: RequestPriority,
}

/// Input for updating a request (all fields optional)
#[derive(Debug, Deserialize)]
pub struct UpdateRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<RequestStatus>,
    pub priority: Option<RequestPriority>,
}

/// Filters for listing requests
#[derive(Debug, Deserialize)]
pub struct RequestFilters {
    pub status: Option<String>,
    pub category: Option<String>,
    pub user_id: Option<Uuid>,
}

impl Request {
    /// Find a request by ID
    pub async fn find_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<Request>, AppError> {
        sqlx::query_as!(
            Request,
            r#"
            SELECT id, user_id, title, description,
                   category as "category: RequestCategory",
                   status as "status: RequestStatus",
                   priority as "priority: RequestPriority",
                   created_at, updated_at
            FROM requests
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)
    }

    /// Find all requests for a specific user
    pub async fn find_by_user_id(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Request>, AppError> {
        sqlx::query_as!(
            Request,
            r#"
            SELECT id, user_id, title, description,
                   category as "category: RequestCategory",
                   status as "status: RequestStatus",
                   priority as "priority: RequestPriority",
                   created_at, updated_at
            FROM requests
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    /// Create a new request
    pub async fn create(
        pool: &PgPool,
        request: CreateRequest,
    ) -> Result<Request, AppError> {
        let id = Uuid::new_v4();

        // Validate title length
        if request.title.len() > 255 {
            return Err(AppError::BadRequest(
                "Title must be 255 characters or less".to_string(),
            ));
        }

        // Validate description length
        if let Some(desc) = &request.description {
            if desc.len() > 5000 {
                return Err(AppError::BadRequest(
                    "Description must be 5000 characters or less".to_string(),
                ));
            }
        }

        sqlx::query_as!(
            Request,
            r#"
            INSERT INTO requests (id, user_id, title, description, category, status, priority)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_id, title, description,
                      category as "category: RequestCategory",
                      status as "status: RequestStatus",
                      priority as "priority: RequestPriority",
                      created_at, updated_at
            "#,
            id,
            request.user_id,
            request.title,
            request.description,
            request.category as RequestCategory,
            RequestStatus::Open as RequestStatus,
            request.priority as RequestPriority
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    /// Update a request
    /// Creates audit logs for status and priority changes
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        request: UpdateRequest,
        changed_by: Uuid,
    ) -> Result<Request, AppError> {
        // Get the existing request first
        let existing = Self::find_by_id(pool, id).await?.ok_or_else(|| {
            AppError::NotFound(format!("Request {} not found", id))
        })?;

        // Build the update query dynamically based on what fields are provided
        let mut query = String::from("UPDATE requests SET ");
        let mut updates = Vec::new();
        let mut param_index = 2; // Start at $2 because $1 is the id

        if let Some(title) = &request.title {
            if title.len() > 255 {
                return Err(AppError::BadRequest(
                    "Title must be 255 characters or less".to_string(),
                ));
            }
            updates.push(format!("title = ${}", param_index));
            param_index += 1;
        }

        if let Some(description) = &request.description {
            if description.len() > 5000 {
                return Err(AppError::BadRequest(
                    "Description must be 5000 characters or less".to_string(),
                ));
            }
            updates.push(format!("description = ${}", param_index));
            param_index += 1;
        }

        if let Some(status) = &request.status {
            updates.push(format!("status = ${}", param_index));
            param_index += 1;

            // Create audit log for status change
            if existing.status != *status {
                AuditLog::create(
                    pool,
                    id,
                    changed_by,
                    AuditAction::StatusChanged,
                    existing.status.clone().into(),
                    status.to_string().into(),
                )
                .await?;
            }
        }

        if let Some(priority) = &request.priority {
            updates.push(format!("priority = ${}", param_index));
            param_index += 1;

            // Create audit log for priority change
            if existing.priority != *priority {
                AuditLog::create(
                    pool,
                    id,
                    changed_by,
                    AuditAction::Updated,
                    existing.priority.into(),
                    priority.clone().into(),
                )
                .await?;
            }
        }

        if updates.is_empty() {
            // Nothing to update, return existing
            return Ok(existing);
        }

        updates.push(format!("updated_at = ${}", param_index));
        query.push_str(&updates.join(", "));
        query.push_str(&format!(
            " WHERE id = $1 RETURNING id, user_id, title, description, "
        ));
        query.push_str("category as \"category: RequestCategory\", ");
        query.push_str("status as \"status: RequestStatus\", ");
        query.push_str("priority as \"priority: RequestPriority\", ");
        query.push_str("created_at, updated_at");

        // Execute the dynamic query
        let mut query_builder = sqlx::query_as::<_, Request>(&query);

        query_builder = query_builder.bind(id);

        if let Some(title) = &request.title {
            query_builder = query_builder.bind(title);
        }
        if let Some(description) = &request.description {
            query_builder = query_builder.bind(description);
        }
        if let Some(status) = &request.status {
            query_builder = query_builder.bind(status);
        }
        if let Some(priority) = &request.priority {
            query_builder = query_builder.bind(priority);
        }

        // Bind updated_at timestamp
        query_builder = query_builder.bind(Utc::now());

        query_builder.fetch_one(pool).await.map_err(AppError::from)
    }

    /// Delete a request
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        sqlx::query!("DELETE FROM requests WHERE id = $1", id)
            .execute(pool)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    /// List requests with optional filters
    pub async fn list(
        pool: &PgPool,
        filters: RequestFilters,
    ) -> Result<Vec<Request>, AppError> {
        let mut query = String::from(
            r#"
            SELECT id, user_id, title, description,
                   category as "category: RequestCategory",
                   status as "status: RequestStatus",
                   priority as "priority: RequestPriority",
                   created_at, updated_at
            FROM requests
            WHERE 1=1
            "#,
        );

        let mut param_index = 1;

        if filters.user_id.is_some() {
            query.push_str(&format!(" AND user_id = ${}", param_index));
            param_index += 1;
        }

        if filters.status.is_some() {
            query.push_str(&format!(" AND status = ${}", param_index));
            param_index += 1;
        }

        if filters.category.is_some() {
            query.push_str(&format!(" AND category = ${}", param_index));
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut query_builder = sqlx::query_as::<_, Request>(&query);

        if let Some(user_id) = filters.user_id {
            query_builder = query_builder.bind(user_id);
        }

        if let Some(status_str) = filters.status {
            let status = RequestStatus::from_str(&status_str)
                .map_err(|e| AppError::BadRequest(e))?;
            query_builder = query_builder.bind(status);
        }

        if let Some(category_str) = filters.category {
            let category = RequestCategory::from_str(&category_str)
                .map_err(|e| AppError::BadRequest(e))?;
            query_builder = query_builder.bind(category);
        }

        query_builder.fetch_all(pool).await.map_err(AppError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_status_from_str() {
        assert_eq!(
            RequestStatus::from_str("open").unwrap(),
            RequestStatus::Open
        );
        assert_eq!(
            RequestStatus::from_str("in_progress").unwrap(),
            RequestStatus::InProgress
        );
        assert_eq!(
            RequestStatus::from_str("resolved").unwrap(),
            RequestStatus::Resolved
        );
        assert!(RequestStatus::from_str("invalid").is_err());
    }

    #[test]
    fn test_request_category_from_str() {
        assert_eq!(
            RequestCategory::from_str("IT").unwrap(),
            RequestCategory::IT
        );
        assert_eq!(
            RequestCategory::from_str("ops").unwrap(),
            RequestCategory::Ops
        );
        assert_eq!(
            RequestCategory::from_str("admin").unwrap(),
            RequestCategory::Admin
        );
        assert_eq!(
            RequestCategory::from_str("HR").unwrap(),
            RequestCategory::HR
        );
        assert!(RequestCategory::from_str("invalid").is_err());
    }

    #[test]
    fn test_request_priority_from_str() {
        assert_eq!(
            RequestPriority::from_str("low").unwrap(),
            RequestPriority::Low
        );
        assert_eq!(
            RequestPriority::from_str("medium").unwrap(),
            RequestPriority::Medium
        );
        assert_eq!(
            RequestPriority::from_str("high").unwrap(),
            RequestPriority::High
        );
        assert!(RequestPriority::from_str("invalid").is_err());
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_request() {
        let pool = setup_test_pool().await;

        let user_id = Uuid::new_v4();
        let request = Request::create(
            &pool,
            CreateRequest {
                user_id,
                title: "Test Request".to_string(),
                description: Some("Test Description".to_string()),
                category: RequestCategory::IT,
                priority: RequestPriority::Medium,
            },
        )
        .await
        .unwrap();

        assert_eq!(request.user_id, Some(user_id));
        assert_eq!(request.title, "Test Request");
        assert_eq!(request.status, RequestStatus::Open);
        assert_eq!(request.category, RequestCategory::IT);
    }

    #[tokio::test]
    #[ignore]
    async fn test_update_request_status() {
        let pool = setup_test_pool().await;

        let user_id = Uuid::new_v4();
        let request = Request::create(
            &pool,
            CreateRequest {
                user_id,
                title: "Test Request".to_string(),
                description: None,
                category: RequestCategory::IT,
                priority: RequestPriority::Medium,
            },
        )
        .await
        .unwrap();

        let updated = Request::update(
            &pool,
            request.id,
            UpdateRequest {
                status: Some(RequestStatus::InProgress),
                ..Default::default()
            },
            user_id,
        )
        .await
        .unwrap();

        assert_eq!(updated.status, RequestStatus::InProgress);
    }

    async fn setup_test_pool() -> PgPool {
        panic!("Test database not configured");
    }
}

// Implement Default for UpdateRequest
impl Default for UpdateRequest {
    fn default() -> Self {
        UpdateRequest {
            title: None,
            description: None,
            status: None,
            priority: None,
        }
    }
}
