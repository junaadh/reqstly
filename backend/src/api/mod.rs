use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{
    AppState,
    auth::{AuthUser, authenticate},
    error::{AppError, ErrorDetail},
    response,
};

#[derive(Debug, Serialize, FromRow)]
struct AuthUserRow {
    id: Uuid,
    email: String,
    display_name: String,
}

#[derive(Debug, Serialize, FromRow)]
struct RequestRow {
    id: Uuid,
    owner_user_id: Uuid,
    title: String,
    description: Option<String>,
    category: String,
    status: String,
    priority: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
struct AuditLogRow {
    id: Uuid,
    request_id: Uuid,
    actor_user_id: Uuid,
    action: String,
    old_value: serde_json::Value,
    new_value: serde_json::Value,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct ListRequestsQuery {
    status: Option<String>,
    category: Option<String>,
    priority: Option<String>,
    sort: Option<String>,
    page: Option<u64>,
    limit: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CreateRequestInput {
    title: String,
    description: Option<String>,
    category: String,
    priority: String,
}

#[derive(Debug, Deserialize)]
struct UpdateRequestInput {
    title: Option<String>,
    description: Option<String>,
    category: Option<String>,
    status: Option<String>,
    priority: Option<String>,
}

#[derive(Debug, Serialize)]
struct MeResponse {
    id: Uuid,
    email: String,
    display_name: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/me", get(me))
        .route("/meta/enums", get(get_enums))
        .route("/requests", get(list_requests).post(create_request))
        .route(
            "/requests/:id",
            get(get_request)
                .patch(update_request)
                .delete(delete_request),
        )
        .route("/requests/:id/audit", get(get_request_audit))
}

pub async fn health(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await?;

    Ok(response::ok(
        StatusCode::OK,
        json!({
            "status": "ok",
            "service": "reqstly_backend",
            "version": "0.2.0"
        }),
    ))
}

async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let auth = authenticate(&headers, &state.jwt_secret, &state.jwt_issuer)?;
    let user = fetch_auth_user(&state.db, &auth).await?;

    Ok(response::ok(
        StatusCode::OK,
        MeResponse {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
        },
    ))
}

async fn get_enums() -> impl IntoResponse {
    response::ok(
        StatusCode::OK,
        json!({
            "status": ["open", "in_progress", "resolved"],
            "category": ["IT", "Ops", "Admin", "HR"],
            "priority": ["low", "medium", "high"]
        }),
    )
}

async fn list_requests(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListRequestsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let auth = authenticate(&headers, &state.jwt_secret, &state.jwt_issuer)?;
    let user = fetch_auth_user(&state.db, &auth).await?;

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = ((page - 1) * limit) as i64;

    if let Some(status) = &query.status {
        validate_status(status)?;
    }
    if let Some(category) = &query.category {
        validate_category(category)?;
    }
    if let Some(priority) = &query.priority {
        validate_priority(priority)?;
    }

    let sort_clause = match query.sort.as_deref() {
        Some("created_at") => "created_at ASC",
        Some("updated_at") => "updated_at ASC",
        Some("-updated_at") => "updated_at DESC",
        _ => "created_at DESC",
    };

    let status_filter = query.status.clone();
    let category_filter = query.category.clone();
    let priority_filter = query.priority.clone();

    let list_query = format!(
        "SELECT id, owner_user_id, title, description, category, status, priority, created_at, updated_at
         FROM app.requests
         WHERE owner_user_id = $1
           AND ($2::text IS NULL OR status = $2)
           AND ($3::text IS NULL OR category = $3)
           AND ($4::text IS NULL OR priority = $4)
         ORDER BY {sort_clause}
         LIMIT $5 OFFSET $6"
    );

    let items = sqlx::query_as::<_, RequestRow>(&list_query)
        .bind(user.id)
        .bind(status_filter.clone())
        .bind(category_filter.clone())
        .bind(priority_filter.clone())
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM app.requests
         WHERE owner_user_id = $1
           AND ($2::text IS NULL OR status = $2)
           AND ($3::text IS NULL OR category = $3)
           AND ($4::text IS NULL OR priority = $4)",
    )
    .bind(user.id)
    .bind(status_filter)
    .bind(category_filter)
    .bind(priority_filter)
    .fetch_one(&state.db)
    .await?;

    Ok(response::list(items, page, limit, total as u64))
}

async fn create_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreateRequestInput>,
) -> Result<impl IntoResponse, AppError> {
    validate_create_input(&input)?;

    let auth = authenticate(&headers, &state.jwt_secret, &state.jwt_issuer)?;
    let user = fetch_auth_user(&state.db, &auth).await?;

    let record = sqlx::query_as::<_, RequestRow>(
        "INSERT INTO app.requests (owner_user_id, title, description, category, status, priority)
         VALUES ($1, $2, $3, $4, 'open', $5)
         RETURNING id, owner_user_id, title, description, category, status, priority, created_at, updated_at",
    )
    .bind(user.id)
    .bind(input.title.trim())
    .bind(input.description.as_deref())
    .bind(input.category)
    .bind(input.priority)
    .fetch_one(&state.db)
    .await?;

    sqlx::query(
        "INSERT INTO app.request_audit_logs (request_id, actor_user_id, action, old_value, new_value)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(record.id)
    .bind(user.id)
    .bind("created")
    .bind(json!({}))
    .bind(json!({
        "title": record.title,
        "status": record.status,
        "category": record.category,
        "priority": record.priority,
    }))
    .execute(&state.db)
    .await?;

    Ok(response::ok(StatusCode::CREATED, record))
}

async fn get_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let auth = authenticate(&headers, &state.jwt_secret, &state.jwt_issuer)?;
    let user = fetch_auth_user(&state.db, &auth).await?;

    let item = fetch_owned_request(&state.db, id, user.id).await?;
    Ok(response::ok(StatusCode::OK, item))
}

async fn update_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateRequestInput>,
) -> Result<impl IntoResponse, AppError> {
    let auth = authenticate(&headers, &state.jwt_secret, &state.jwt_issuer)?;
    let user = fetch_auth_user(&state.db, &auth).await?;

    let existing = fetch_owned_request(&state.db, id, user.id).await?;

    if let Some(category) = &input.category {
        validate_category(category)?;
    }
    if let Some(priority) = &input.priority {
        validate_priority(priority)?;
    }
    if let Some(status) = &input.status {
        validate_status(status)?;
    }

    let next_title = input
        .title
        .as_deref()
        .map(str::trim)
        .unwrap_or(existing.title.as_str())
        .to_string();

    if next_title.is_empty() {
        return Err(AppError::Validation(vec![ErrorDetail {
            field: "title".to_string(),
            message: "title cannot be empty".to_string(),
        }]));
    }

    let next_description = input.description.or(existing.description.clone());
    let next_category = input.category.unwrap_or(existing.category.clone());
    let next_status = input.status.unwrap_or(existing.status.clone());
    let next_priority = input.priority.unwrap_or(existing.priority.clone());

    let updated = sqlx::query_as::<_, RequestRow>(
        "UPDATE app.requests
         SET title = $2, description = $3, category = $4, status = $5, priority = $6, updated_at = NOW()
         WHERE id = $1
         RETURNING id, owner_user_id, title, description, category, status, priority, created_at, updated_at",
    )
    .bind(id)
    .bind(&next_title)
    .bind(next_description.as_deref())
    .bind(&next_category)
    .bind(&next_status)
    .bind(&next_priority)
    .fetch_one(&state.db)
    .await?;

    sqlx::query(
        "INSERT INTO app.request_audit_logs (request_id, actor_user_id, action, old_value, new_value)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(updated.id)
    .bind(user.id)
    .bind("updated")
    .bind(json!({
        "title": existing.title,
        "description": existing.description,
        "category": existing.category,
        "status": existing.status,
        "priority": existing.priority,
    }))
    .bind(json!({
        "title": updated.title,
        "description": updated.description,
        "category": updated.category,
        "status": updated.status,
        "priority": updated.priority,
    }))
    .execute(&state.db)
    .await?;

    Ok(response::ok(StatusCode::OK, updated))
}

async fn delete_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let auth = authenticate(&headers, &state.jwt_secret, &state.jwt_issuer)?;
    let user = fetch_auth_user(&state.db, &auth).await?;

    let existing = fetch_owned_request(&state.db, id, user.id).await?;

    sqlx::query(
        "INSERT INTO app.request_audit_logs (request_id, actor_user_id, action, old_value, new_value)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(existing.id)
    .bind(user.id)
    .bind("deleted")
    .bind(json!({
        "title": existing.title,
        "description": existing.description,
        "category": existing.category,
        "status": existing.status,
        "priority": existing.priority,
    }))
    .bind(json!({}))
    .execute(&state.db)
    .await?;

    sqlx::query("DELETE FROM app.requests WHERE id = $1")
        .bind(existing.id)
        .execute(&state.db)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_request_audit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let auth = authenticate(&headers, &state.jwt_secret, &state.jwt_issuer)?;
    let user = fetch_auth_user(&state.db, &auth).await?;

    let _ = fetch_owned_request(&state.db, id, user.id).await?;

    let items = sqlx::query_as::<_, AuditLogRow>(
        "SELECT id, request_id, actor_user_id, action, old_value, new_value, created_at
         FROM app.request_audit_logs
         WHERE request_id = $1
         ORDER BY created_at DESC",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(response::ok(StatusCode::OK, items))
}

async fn fetch_auth_user(
    pool: &PgPool,
    auth: &AuthUser,
) -> Result<AuthUserRow, AppError> {
    let fallback_email = format!("{}@users.reqstly.local", auth.auth_user_id);

    sqlx::query_as::<_, AuthUserRow>(
        "SELECT
            id,
            COALESCE(email::text, $2) AS email,
            COALESCE(
                NULLIF(raw_user_meta_data ->> 'display_name', ''),
                NULLIF(raw_user_meta_data ->> 'full_name', ''),
                split_part(COALESCE(email::text, $2), '@', 1),
                'user'
            ) AS display_name
         FROM auth.users
         WHERE id = $1",
    )
    .bind(auth.auth_user_id)
    .bind(auth.email.clone().unwrap_or_else(|| fallback_email.clone()))
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::Unauthorized("auth user not found".to_string()))
}

async fn fetch_owned_request(
    pool: &PgPool,
    request_id: Uuid,
    owner_id: Uuid,
) -> Result<RequestRow, AppError> {
    sqlx::query_as::<_, RequestRow>(
        "SELECT id, owner_user_id, title, description, category, status, priority, created_at, updated_at
         FROM app.requests
         WHERE id = $1 AND owner_user_id = $2",
    )
    .bind(request_id)
    .bind(owner_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("request not found".to_string()))
}

fn validate_create_input(input: &CreateRequestInput) -> Result<(), AppError> {
    let mut details = Vec::new();

    if input.title.trim().is_empty() {
        details.push(ErrorDetail {
            field: "title".to_string(),
            message: "title is required".to_string(),
        });
    }

    if input.title.len() > 255 {
        details.push(ErrorDetail {
            field: "title".to_string(),
            message: "title must be <= 255 characters".to_string(),
        });
    }

    if let Some(description) = &input.description
        && description.len() > 5000
    {
        details.push(ErrorDetail {
            field: "description".to_string(),
            message: "description must be <= 5000 characters".to_string(),
        });
    }

    if validate_category(&input.category).is_err() {
        details.push(ErrorDetail {
            field: "category".to_string(),
            message: "category must be one of IT, Ops, Admin, HR".to_string(),
        });
    }

    if validate_priority(&input.priority).is_err() {
        details.push(ErrorDetail {
            field: "priority".to_string(),
            message: "priority must be one of low, medium, high".to_string(),
        });
    }

    if details.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(details))
    }
}

fn validate_status(value: &str) -> Result<(), AppError> {
    match value {
        "open" | "in_progress" | "resolved" => Ok(()),
        _ => Err(AppError::Validation(vec![ErrorDetail {
            field: "status".to_string(),
            message: "status must be one of open, in_progress, resolved"
                .to_string(),
        }])),
    }
}

fn validate_category(value: &str) -> Result<(), AppError> {
    match value {
        "IT" | "Ops" | "Admin" | "HR" => Ok(()),
        _ => Err(AppError::Validation(vec![ErrorDetail {
            field: "category".to_string(),
            message: "category must be one of IT, Ops, Admin, HR".to_string(),
        }])),
    }
}

fn validate_priority(value: &str) -> Result<(), AppError> {
    match value {
        "low" | "medium" | "high" => Ok(()),
        _ => Err(AppError::Validation(vec![ErrorDetail {
            field: "priority".to_string(),
            message: "priority must be one of low, medium, high".to_string(),
        }])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_status_rejects_invalid() {
        let err = validate_status("bad").expect_err("status should fail");
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_priority_accepts_valid() {
        validate_priority("low").expect("low should pass");
        validate_priority("medium").expect("medium should pass");
        validate_priority("high").expect("high should pass");
    }

    #[test]
    fn validate_create_input_rejects_empty_title() {
        let input = CreateRequestInput {
            title: " ".to_string(),
            description: None,
            category: "IT".to_string(),
            priority: "low".to_string(),
        };

        let err =
            validate_create_input(&input).expect_err("empty title should fail");
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_category_rejects_invalid() {
        let err = validate_category("Finance").expect_err("invalid category");
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_create_input_rejects_long_description() {
        let input = CreateRequestInput {
            title: "Need access".to_string(),
            description: Some("x".repeat(5001)),
            category: "IT".to_string(),
            priority: "medium".to_string(),
        };

        let err = validate_create_input(&input)
            .expect_err("long description should fail");
        assert!(matches!(err, AppError::Validation(_)));
    }
}
