mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod metrics;
mod models;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use config::{AzureAd, Passkey, Settings};
use db::DbPool;
use redis::Commands;
use serde_json::json;
use std::net::SocketAddr;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use webauthn_rs::{Webauthn, WebauthnBuilder};

use crate::{
    auth::auth_context::AuthContext,
    auth::azure::{AzureOidc, azure_callback, azure_login},
    auth::password::create_password_routes,
    auth::session_token::SessionToken,
    error::AppError,
    handlers::requests::create_request_routes,
    models::Session,
};

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub redis: redis::Client,
    pub azure: AzureAd,
    pub passkey: Passkey,
    pub azure_client: Option<AzureOidc>,
    pub webauthn: Webauthn,
}

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    "reqstly_backend=info,tower_http=debug,axum=trace".into()
                }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let settings = Settings::from_env().expect("Failed to load configuration");

    tracing::info!("Starting Reqstly backend on port {}", settings.server.port);

    // Create database connection pool
    let pool = db::create_pool(&settings.database.url)
        .await
        .expect("Failed to create database pool");

    let redis_client = redis::Client::open(settings.redis.url.as_str())
        .expect("Failed to create redis client");

    // Build authentication configs
    let azure_config = AzureAd {
        client_id: settings.azure_ad.client_id.clone(),
        tenant_id: settings.azure_ad.tenant_id.clone(),
        client_secret: settings.azure_ad.client_secret.clone(),
    };

    let passkey_config = Passkey {
        rp_id: settings.passkey.rp_id.clone(),
        origin: settings.passkey.origin.clone(),
    };

    let azure_client = if azure_config.client_id.is_empty()
        || azure_config.tenant_id.is_empty()
        || azure_config.client_secret.is_empty()
    {
        tracing::warn!(
            "Azure AD config missing; Azure login disabled until set"
        );
        None
    } else {
        Some(
            AzureOidc::new(
                &azure_config,
                format!("{}/auth/azure/callback", &settings.server.base_url),
            )
            .await
            .expect("Failed to create azure client"),
        )
    };

    let rp_origin = webauthn_rs::prelude::Url::parse(&passkey_config.origin)
        .expect("Invalid passkey origin");
    let webauthn_builder =
        WebauthnBuilder::new(&passkey_config.rp_id, &rp_origin)
            .expect("Invalid passkey configuration");
    let webauthn_builder = webauthn_builder.rp_name("Reqstly Passkey");
    let webauthn = webauthn_builder
        .build()
        .expect("Failed to build passkey client");

    let state = AppState {
        db: pool.clone(),
        azure: azure_config,
        passkey: passkey_config,
        redis: redis_client,
        azure_client,
        webauthn,
    };

    let auth_routes = Router::new()
        .route("/azure/login", get(azure_login))
        .route("/azure/callback", get(azure_callback))
        // .route("/passkey/login/start", post(passkey_login_start))
        // .route("/passkey/login/finish", post(passkey_login_finish))
        // .route("/passkey/register/start", post(passkey_register_start))
        // .route("/passkey/register/finish", post(passkey_register_finish))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .nest("/password", create_password_routes());

    // Build our application with routes
    let app = Router::new()
        // Health and metrics (public)
        .route("/health", get(health_check))
        .route("/metrics", get(metrics))
        // Auth routes
        .nest("/auth", auth_routes)
        // Request routes (authenticated)
        .nest("/requests", create_request_routes())
        // Middleware
        .layer(CookieManagerLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::AllowOrigin::list([
                    "https://reqstly.com".parse().unwrap(),
                ]))
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([header::CONTENT_TYPE])
                .allow_credentials(true),
        )
        .layer(TraceLayer::new_for_http())
        // State
        .with_state(state);

    // Create socket address
    let addr = SocketAddr::from(([0, 0, 0, 0], settings.server.port));

    tracing::info!("Listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.expect("Server error");
}

async fn health_check(State(app): State<AppState>) -> Response {
    // Test database connection
    let pg_result = sqlx::query("SELECT 1").fetch_one(&app.db).await;
    let mut redis_conn =
        match app.redis.get_connection().map_err(AppError::from) {
            Ok(conn) => conn,
            Err(err) => return err.into_response(),
        };
    let rd_result: Result<String, _> = redis_conn.ping();

    let status = if pg_result.is_ok() && rd_result.is_ok() {
        "healthy"
    } else {
        "unhealthy"
    };

    (
        StatusCode::OK,
        Json(json!({
            "status": status,
            "service": "reqstly_backend",
            "version": "0.1.0"
        })),
    )
        .into_response()
}

async fn logout(
    _auth: AuthContext,
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<Response, AppError> {
    let session_cookie = cookies
        .get("session")
        .ok_or(AppError::BadRequest("No session cookie found".to_string()))?;

    Session::invalidate(
        &state.db,
        &SessionToken::new(session_cookie.value().to_string()),
    )
    .await?;

    let mut cookie = Cookie::from("session");
    cookie.set_path("/");
    cookie.set_max_age(tower_cookies::cookie::time::Duration::ZERO);
    cookies.add(cookie);

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Logged out successfully"
        })),
    )
        .into_response())
}

async fn me(auth: AuthContext) -> impl IntoResponse {
    Json(json!({
        "id": auth.user.id,
        "email": auth.user.email,
        "name": auth.user.name,
        "provider": auth.provider().to_string(),
        "federated": auth.is_federated()
    }))
}

async fn metrics() -> impl IntoResponse {
    // Basic Prometheus metrics placeholder
    // In a full implementation, this would return actual Prometheus metrics
    (
        StatusCode::OK,
        "# HELP reqstly_backend_info Information about the backend
# TYPE reqstly_backend_info gauge
reqstly_backend_info{version=\"0.1.0\"} 1
",
    )
}
