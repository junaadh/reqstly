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
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use config::{AzureAd, Passkey, Settings};
use db::DbPool;
use models::User;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_cookies::CookieManagerLayer;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    // Run migrations
    // tracing::info!("Running database migrations");
    // db::run_migrations(&pool)
    //     .await
    //     .expect("Failed to run migrations");

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

    // Build our application with routes
    let app = Router::new()
        // Health and metrics (public)
        .route("/health", get(health_check))
        .route("/metrics", get(metrics))
        // Auth: Azure AD (public)
        // .route("/auth/azure/login", get(azure_login))
        // .route("/auth/azure/callback", get(azure_callback))
        // Auth: Passkey authentication (public)
        // .route("/auth/passkey/login/start", post(passkey_login_start))
        // .route("/auth/passkey/login/finish", post(passkey_login_finish))
        // // Auth: Passkey registration (protected - requires auth)
        // .route("/auth/passkey/register/start", post(passkey_register_start))
        // .route(
        //     "/auth/passkey/register/finish",
        //     post(passkey_register_finish),
        // )
        // Auth: Logout (protected)
        // .route("/auth/logout", post(logout))
        // // Auth: Get current user (protected)
        // .route("/auth/me", get(me))
        // Middleware
        .layer(CookieManagerLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        // State
        .with_state(pool.clone())
        .with_state(azure_config)
        .with_state(passkey_config);

    // Add DbPool to request extensions for middleware access
    // let app = app.layer(axum::middleware::from_fn(
    //     |mut req: axum::extract::Request, next: axum::middleware::Next| async move {
    //         // Clone pool for this request
    //         let pool_clone = pool.clone();
    //         // Add to request extensions
    //         req.extensions_mut().insert(pool_clone);
    //         // Continue to next handler
    //         Ok(next.run(req).await)
    //     },
    // ));

    // Create socket address
    let addr = SocketAddr::from(([0, 0, 0, 0], settings.server.port));

    tracing::info!("Listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.expect("Server error");
}

async fn health_check(State(pool): State<DbPool>) -> impl IntoResponse {
    // Test database connection
    let result = sqlx::query("SELECT 1").fetch_one(&pool).await;

    let status = if result.is_ok() {
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
