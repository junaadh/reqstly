use crate::{
    auth::auth_context::AuthContext,
    models::{
        audit_log::AuditAction,
        request::{CreateRequest, Request, UpdateRequest},
        AuditLog,
    },
    AppState, error::AppError,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use serde::Deserialize;
use uuid::Uuid;

/// Query parameters for listing requests
#[derive(Debug, Deserialize)]
pub struct RequestQueryParams {
    status: Option<String>,
    category: Option<String>,
}

/// Create a new request
/// POST /requests
pub async fn create_request(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(input): Json<CreateRequestRequest>,
) -> Result<Response, AppError> {
    // Validate input
    if input.title.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Title cannot be empty".to_string(),
        ));
    }

    // Create the request with authenticated user's ID
    let request = Request::create(
        &state.db,
        CreateRequest {
            user_id: auth.user.id,
            title: input.title,
            description: input.description,
            category: input.category,
            priority: input.priority,
        },
    )
    .await?;

    // Create audit log
    AuditLog::create(
        &state.db,
        request.id,
        auth.user.id,
        AuditAction::Created,
        serde_json::Value::Null,
        serde_json::json!({
            "title": request.title,
            "category": request.category.to_string(),
            "priority": request.priority.to_string(),
        }),
    )
    .await?;

    tracing::info!(
        "Request created by user {}: {}",
        auth.user.email,
        request.id
    );

    Ok((
        StatusCode::CREATED,
        Json(request),
    )
        .into_response())
}

/// List requests with optional filters
/// GET /requests?status=open&category=IT
pub async fn list_requests(
    auth: AuthContext,
    State(state): State<AppState>,
    Query(params): Query<RequestQueryParams>,
) -> Result<Response, AppError> {
    let filters = crate::models::request::RequestFilters {
        status: params.status,
        category: params.category,
        user_id: Some(auth.user.id), // Only show user's own requests
    };

    let requests = Request::list(&state.db, filters).await?;

    Ok(Json(requests).into_response())
}

/// Get a specific request by ID
/// GET /requests/:id
pub async fn get_request(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let request = Request::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Request {} not found", id)))?;

    // Check authorization: user can only view their own requests
    if request.user_id != Some(auth.user.id) {
        return Err(AppError::Forbidden(
            "You can only view your own requests".to_string(),
        ));
    }

    Ok(Json(request).into_response())
}

/// Update a request
/// PUT /requests/:id
pub async fn update_request(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateRequest>,
) -> Result<Response, AppError> {
    // Check if request exists and belongs to user
    let existing = Request::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Request {} not found", id)))?;

    if existing.user_id != Some(auth.user.id) {
        return Err(AppError::Forbidden(
            "You can only modify your own requests".to_string(),
        ));
    }

    // Update the request
    let updated = Request::update(&state.db, id, input, auth.user.id).await?;

    tracing::info!(
        "Request {} updated by user {}",
        id,
        auth.user.email
    );

    Ok(Json(updated).into_response())
}

/// Delete a request
/// DELETE /requests/:id
pub async fn delete_request(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    // Check if request exists and belongs to user
    let request = Request::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Request {} not found", id)))?;

    if request.user_id != Some(auth.user.id) {
        return Err(AppError::Forbidden(
            "You can only delete your own requests".to_string(),
        ));
    }

    // Create audit log before deletion
    AuditLog::create(
        &state.db,
        id,
        auth.user.id,
        AuditAction::Deleted,
        serde_json::json!({
            "title": request.title,
            "status": request.status.to_string(),
        }),
        serde_json::Value::Null,
    )
    .await?;

    // Delete the request (cascade will delete audit logs)
    Request::delete(&state.db, id).await?;

    tracing::info!(
        "Request {} deleted by user {}",
        id,
        auth.user.email
    );

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Get audit log for a specific request
/// GET /requests/:id/audit
pub async fn get_request_audit(
    auth: AuthContext,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    // Check if request exists and belongs to user
    let request = Request::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Request {} not found", id)))?;

    if request.user_id != Some(auth.user.id) {
        return Err(AppError::Forbidden(
            "You can only view audit logs for your own requests".to_string(),
        ));
    }

    let audit_logs = AuditLog::find_by_request_id(&state.db, id).await?;

    Ok(Json(audit_logs).into_response())
}

/// Request creation input from HTTP request
#[derive(Debug, Deserialize)]
struct CreateRequestRequest {
    title: String,
    description: Option<String>,
    category: crate::models::request::RequestCategory,
    priority: crate::models::request::RequestPriority,
}

/// Create request routes
pub fn create_request_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_request).get(list_requests))
        .route("/:id", get(get_request).put(update_request).delete(delete_request))
        .route("/:id/audit", get(get_request_audit))
}
