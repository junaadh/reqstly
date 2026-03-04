pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod response;
pub mod telemetry;

use axum::{
    Router,
    extract::Request,
    http::{HeaderValue, Method},
    routing::get,
};
use sqlx::PgPool;
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{
        DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
    },
};
use tracing::Level;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
    pub jwt_issuer: String,
}

pub fn build_app(
    state: AppState,
    allowed_origin: &str,
) -> Result<Router, error::AppError> {
    let cors_layer = if allowed_origin == "*" {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers(Any)
    } else {
        let parsed: HeaderValue = allowed_origin.parse().map_err(|_| {
            error::AppError::Internal("invalid CORS origin".to_string())
        })?;

        CorsLayer::new()
            .allow_origin(parsed)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers(Any)
    };

    Ok(Router::new()
        .route("/health", get(api::health))
        .nest("/api/v1", api::router())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request| {
                    let request_id = request
                        .headers()
                        .get("x-request-id")
                        .and_then(|value| value.to_str().ok())
                        .unwrap_or("missing")
                        .to_string();
                    let user_agent = request
                        .headers()
                        .get("user-agent")
                        .and_then(|value| value.to_str().ok())
                        .unwrap_or("unknown")
                        .to_string();

                    tracing::info_span!(
                        "http.request",
                        http.method = %request.method(),
                        url.path = %request.uri().path(),
                        http.flavor = ?request.version(),
                        http.request_id = %request_id,
                        user_agent = %user_agent
                    )
                })
                .on_request(DefaultOnRequest::new().level(Level::DEBUG))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(cors_layer)
        .with_state(state))
}
