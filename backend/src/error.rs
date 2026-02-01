use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use thiserror::Error;

/// Centralized application error type
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Database(err) => {
                // Map specific database errors to appropriate HTTP status codes
                match &err {
                    sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
                    sqlx::Error::Database(db_err) => {
                        if let Some(code) = db_err.code() {
                            match code.as_ref() {
                                "23505" => (StatusCode::CONFLICT, "Resource already exists".to_string()),
                                "23503" => (StatusCode::BAD_REQUEST, "Referenced resource doesn't exist".to_string()),
                                "23502" => (StatusCode::BAD_REQUEST, "Missing required field".to_string()),
                                _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", db_err)),
                            }
                        } else {
                            (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", db_err))
                        }
                    }
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", err)),
                }
            }
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        tracing::error!("AppError: {} - {}", status, message);

        (
            status,
            Json(json!({
                "error": message
            }))
        ).into_response()
    }
}
