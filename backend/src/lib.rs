pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod realtime;
pub mod response;
pub mod telemetry;

use axum::{
    Router,
    extract::Request,
    http::{
        HeaderValue, Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderName},
    },
    routing::get,
};
use axum_prometheus::PrometheusMetricLayer;
use sqlx::PgPool;
use std::sync::{Arc, OnceLock};
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{
        DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
    },
};
use tracing::Level;

type MetricsRenderer = Arc<dyn Fn() -> String + Send + Sync>;

static METRICS: OnceLock<(PrometheusMetricLayer<'static>, MetricsRenderer)> =
    OnceLock::new();

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub ws_token_secret: String,
    pub ws_token_issuer: String,
    pub passkey: auth::PasskeyService,
    pub realtime_hub: realtime::RealtimeHub,
    pub ws_allowed_origins: Vec<String>,
}

pub fn build_app(
    state: AppState,
    allowed_origin: &str,
) -> Result<Router, error::AppError> {
    let (metrics_layer, metrics_renderer) = METRICS
        .get_or_init(|| {
            let (layer, handle) = PrometheusMetricLayer::pair();
            let renderer: MetricsRenderer = Arc::new(move || handle.render());
            (layer, renderer)
        })
        .clone();

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
            .allow_credentials(false)
    } else {
        let parsed: HeaderValue = allowed_origin.parse().map_err(|_| {
            error::AppError::Internal("invalid CORS origin".to_string())
        })?;
        let x_request_id = HeaderName::from_static("x-request-id");
        let x_csrf_token = HeaderName::from_static("x-csrf-token");

        CorsLayer::new()
            .allow_origin(parsed)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                ACCEPT,
                AUTHORIZATION,
                CONTENT_TYPE,
                x_request_id,
                x_csrf_token,
            ])
            .allow_credentials(true)
    };

    Ok(Router::new()
        .route("/health", get(api::health))
        .route(
            "/metrics",
            get(move || {
                let metrics_renderer = metrics_renderer.clone();
                async move { (metrics_renderer)() }
            }),
        )
        .route("/ws", get(api::ws))
        .nest("/api/v1", api::router())
        .layer(metrics_layer)
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
