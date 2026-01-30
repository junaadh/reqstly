mod config;
mod db;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json,
};
use db::DbPool;
use serde_json::json;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "reqstly_backend=info,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let settings = config::Settings::from_env()
        .expect("Failed to load configuration");

    tracing::info!("Starting Reqstly backend on port {}", settings.server.port);

    // Create database connection pool
    let pool = db::create_pool(&settings.database.url)
        .await
        .expect("Failed to create database pool");

    // Run migrations
    tracing::info!("Running database migrations");
    db::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    // Build our application with routes
    let app = axum::Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics))
        .with_state(pool)
        .layer(TraceLayer::new_for_http());

    // Create socket address
    let addr = SocketAddr::from(([0, 0, 0, 0], settings.server.port));

    tracing::info!("Listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

async fn health_check(State(pool): State<DbPool>) -> impl IntoResponse {
    // Test database connection
    let result = sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await;

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
