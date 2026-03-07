#![allow(dead_code)]

use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use http_body_util::BodyExt;
use jsonwebtoken::{EncodingKey, Header, encode};
use reqstly_backend::{AppState, build_app, db};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use tower::util::ServiceExt;
use tower_sessions::{MemoryStore, SessionManagerLayer};
use url::Url;
use uuid::Uuid;

pub const TEST_WS_TOKEN_SECRET: &str = "integration-test-secret";
pub const TEST_WS_TOKEN_ISSUER: &str = "reqstly.test/ws";
const DEFAULT_ADMIN_DATABASE_URL: &str = "postgres://postgres:super-secret-and-long-postgres-password@127.0.0.1:54323/postgres";

#[derive(Debug)]
pub struct TestContext {
    pub app: axum::Router,
    pub pool: PgPool,
    pub admin_database_url: String,
    pub db_name: String,
    pub user_id: Uuid,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct TestClaims {
    pub sub: String,
    pub email: String,
    pub aud: String,
    pub iss: String,
    pub exp: usize,
}

impl TestContext {
    pub async fn new() -> Self {
        let admin_database_url = std::env::var("TEST_DATABASE_ADMIN_URL")
            .unwrap_or_else(|_| DEFAULT_ADMIN_DATABASE_URL.to_string());

        let db_name = format!("reqstly_it_{}", Uuid::new_v4().simple());
        let app_database_url =
            create_test_database(&admin_database_url, &db_name).await;

        let pool = PgPool::connect(&app_database_url)
            .await
            .expect("test db should connect");

        db::run_migrations(&pool)
            .await
            .expect("migrations should apply in test db");

        let user_id = Uuid::new_v4();
        insert_auth_user(&pool, user_id, "qa@example.com", "qa-user").await;

        let token = build_token(user_id);
        insert_ws_token_issuance(&pool, user_id, &token).await;
        let app = build_app(
            AppState {
                db: pool.clone(),
                ws_token_secret: TEST_WS_TOKEN_SECRET.to_string(),
                ws_token_issuer: TEST_WS_TOKEN_ISSUER.to_string(),
                passkey: reqstly_backend::auth::PasskeyService::new(
                    "localhost",
                    "https://localhost",
                    "Reqstly Integration Tests",
                )
                .expect("passkey service should initialize"),
                realtime_hub: reqstly_backend::realtime::RealtimeHub::new(),
                ws_allowed_origins: vec!["*".to_string()],
            },
            "*",
        )
        .expect("router should build")
        .layer(
            SessionManagerLayer::new(MemoryStore::default()).with_secure(false),
        );

        Self {
            app,
            pool,
            admin_database_url,
            db_name,
            user_id,
            token,
        }
    }

    pub async fn cleanup(self) {
        self.pool.close().await;
        drop_test_database(&self.admin_database_url, &self.db_name).await;
    }
}

pub async fn send_json(
    app: &axum::Router,
    method: Method,
    path: &str,
    bearer_token: Option<&str>,
    payload: Option<Value>,
) -> (StatusCode, Value) {
    let has_payload = payload.is_some();
    let body = match payload {
        Some(value) => Body::from(
            serde_json::to_vec(&value).expect("payload should serialize"),
        ),
        None => Body::empty(),
    };

    let mut request = Request::builder().method(method).uri(path);
    if bearer_token.is_some() || has_payload {
        request = request.header(header::CONTENT_TYPE, "application/json");
    }
    if let Some(token) = bearer_token {
        request =
            request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }

    let response = app
        .clone()
        .oneshot(request.body(body).expect("request should build"))
        .await
        .expect("request should execute");

    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("response body should collect")
        .to_bytes();

    if bytes.is_empty() {
        return (status, json!({}));
    }

    let body_json: Value =
        serde_json::from_slice(&bytes).expect("response should be json");
    (status, body_json)
}

pub async fn create_request(
    ctx: &TestContext,
    title: &str,
    category: &str,
    priority: &str,
) -> (StatusCode, Value) {
    send_json(
        &ctx.app,
        Method::POST,
        "/api/v1/requests",
        Some(&ctx.token),
        Some(json!({
            "title": title,
            "description": null,
            "category": category,
            "priority": priority
        })),
    )
    .await
}

pub fn build_token(user_id: Uuid) -> String {
    build_token_with_claims(
        TestClaims {
            sub: user_id.to_string(),
            email: "qa@example.com".to_string(),
            aud: "ws".to_string(),
            iss: TEST_WS_TOKEN_ISSUER.to_string(),
            exp: 9_999_999_999,
        },
        TEST_WS_TOKEN_SECRET,
    )
}

pub fn build_token_with_claims(claims: TestClaims, secret: &str) -> String {
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("token should encode")
}

pub async fn insert_auth_user(
    pool: &PgPool,
    user_id: Uuid,
    email: &str,
    display_name: &str,
) {
    sqlx::query(
        "INSERT INTO app.app_users (id, email, display_name)
         VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(email)
    .bind(display_name)
    .execute(pool)
    .await
    .expect("auth user insert should succeed");
}

async fn create_test_database(
    admin_database_url: &str,
    db_name: &str,
) -> String {
    let admin_pool = PgPool::connect(admin_database_url)
        .await
        .expect("admin db should connect");

    sqlx::query(&format!("CREATE DATABASE \"{db_name}\""))
        .execute(&admin_pool)
        .await
        .expect("test db should be created");
    admin_pool.close().await;

    let mut app_db_url = Url::parse(admin_database_url)
        .expect("admin database url should parse");
    app_db_url.set_path(&format!("/{db_name}"));
    app_db_url.to_string()
}

async fn drop_test_database(admin_database_url: &str, db_name: &str) {
    let admin_pool = PgPool::connect(admin_database_url)
        .await
        .expect("admin db should connect for cleanup");

    sqlx::query(
        "SELECT pg_terminate_backend(pid)
         FROM pg_stat_activity
         WHERE datname = $1
           AND pid <> pg_backend_pid()",
    )
    .bind(db_name)
    .execute(&admin_pool)
    .await
    .expect("active test db sessions should terminate");

    sqlx::query(&format!("DROP DATABASE IF EXISTS \"{db_name}\""))
        .execute(&admin_pool)
        .await
        .expect("test db should drop");

    admin_pool.close().await;
}

pub async fn insert_ws_token_issuance(
    pool: &PgPool,
    user_id: Uuid,
    token: &str,
) {
    let token_fingerprint = Sha256::digest(token.as_bytes()).to_vec();
    let expires_at = OffsetDateTime::now_utc() + Duration::hours(1);

    sqlx::query(
        "INSERT INTO app.ws_token_issuances
           (user_id, token_fingerprint, expires_at, metadata)
         VALUES ($1, $2, $3, '{}'::jsonb)",
    )
    .bind(user_id)
    .bind(token_fingerprint)
    .bind(expires_at)
    .execute(pool)
    .await
    .expect("test ws token issuance insert should succeed");
}
