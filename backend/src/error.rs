use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct ErrorDetail {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("rate limited: {0}")]
    RateLimited(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("validation failed")]
    Validation(Vec<ErrorDetail>),
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("migration error")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Database(_) | Self::Migration(_) | Self::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::RateLimited(_) => "RATE_LIMITED",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Migration(_) => "MIGRATION_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    fn message(&self) -> String {
        match self {
            Self::Validation(_) => "Validation failed".to_string(),
            Self::Database(_) => "A database error occurred".to_string(),
            Self::Migration(_) => "A migration error occurred".to_string(),
            _ => self.to_string(),
        }
    }
}

#[derive(Serialize)]
struct ErrorEnvelope {
    error: ErrorPayload,
    meta: ErrorMeta,
}

#[derive(Serialize)]
struct ErrorPayload {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Vec<ErrorDetail>>,
}

#[derive(Serialize)]
struct ErrorMeta {
    request_id: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let details = match &self {
            AppError::Validation(items) => Some(items.clone()),
            _ => None,
        };

        match &self {
            AppError::Unauthorized(_)
            | AppError::RateLimited(_)
            | AppError::NotFound(_)
            | AppError::Validation(_) => {
                tracing::warn!("request failed: {}", self);
            }
            AppError::Database(_)
            | AppError::Migration(_)
            | AppError::Internal(_) => {
                tracing::error!("request failed: {}", self);
            }
        }

        (
            status,
            Json(ErrorEnvelope {
                error: ErrorPayload {
                    code: self.code().to_string(),
                    message: self.message(),
                    details,
                },
                meta: ErrorMeta {
                    request_id: Uuid::new_v4().to_string(),
                },
            }),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn validation_error_maps_to_422_and_details() {
        let response = AppError::Validation(vec![ErrorDetail {
            field: "title".to_string(),
            message: "required".to_string(),
        }])
        .into_response();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body should collect")
            .to_bytes();
        let payload: serde_json::Value =
            serde_json::from_slice(&bytes).expect("json body expected");

        assert_eq!(payload["error"]["code"], "VALIDATION_ERROR");
        assert!(payload["error"]["details"].is_array());
        assert!(payload["meta"]["request_id"].is_string());
    }

    #[tokio::test]
    async fn not_found_error_maps_to_404() {
        let response =
            AppError::NotFound("request not found".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
