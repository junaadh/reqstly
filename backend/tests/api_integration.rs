use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use http_body_util::BodyExt;
use jsonwebtoken::{EncodingKey, Header, encode};
use reqstly_backend::{AppState, build_app, db};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::{Executor, PgPool};
use tower::util::ServiceExt;
use url::Url;
use uuid::Uuid;

const TEST_JWT_SECRET: &str = "integration-test-secret";
const TEST_JWT_ISSUER: &str = "https://supabase.localhost/auth/v1";
const DEFAULT_ADMIN_DATABASE_URL: &str = "postgres://postgres:super-secret-and-long-postgres-password@127.0.0.1:54323/postgres";

#[derive(Debug)]
struct TestContext {
    app: axum::Router,
    pool: PgPool,
    admin_database_url: String,
    db_name: String,
    user_id: Uuid,
    token: String,
}

#[derive(Debug, Serialize)]
struct TestClaims {
    sub: String,
    email: String,
    aud: String,
    iss: String,
    exp: usize,
}

impl TestContext {
    async fn new() -> Self {
        let admin_database_url = std::env::var("TEST_DATABASE_ADMIN_URL")
            .unwrap_or_else(|_| DEFAULT_ADMIN_DATABASE_URL.to_string());

        let db_name = format!("reqstly_it_{}", Uuid::new_v4().simple());
        let app_database_url =
            create_test_database(&admin_database_url, &db_name).await;

        let pool = PgPool::connect(&app_database_url)
            .await
            .expect("test db should connect");

        bootstrap_supabase_compat(&pool).await;
        db::run_migrations(&pool)
            .await
            .expect("migrations should apply in test db");

        let user_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO auth.users (id, email, raw_user_meta_data)
             VALUES ($1, $2, $3)",
        )
        .bind(user_id)
        .bind("qa@example.com")
        .bind(json!({
            "display_name": "qa-user"
        }))
        .execute(&pool)
        .await
        .expect("auth user insert should succeed");

        let token = build_token(user_id);
        let app = build_app(
            AppState {
                db: pool.clone(),
                jwt_secret: TEST_JWT_SECRET.to_string(),
                jwt_issuer: TEST_JWT_ISSUER.to_string(),
            },
            "*",
        )
        .expect("router should build");

        Self {
            app,
            pool,
            admin_database_url,
            db_name,
            user_id,
            token,
        }
    }

    async fn cleanup(self) {
        self.pool.close().await;
        drop_test_database(&self.admin_database_url, &self.db_name).await;
    }
}

#[tokio::test]
async fn me_requires_authorization_header() {
    let ctx = TestContext::new().await;

    let (status, payload) =
        send_json(&ctx.app, Method::GET, "/api/v1/me", None, None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(payload["error"]["code"], "UNAUTHORIZED");
    assert!(payload["meta"]["request_id"].is_string());

    ctx.cleanup().await;
}

#[tokio::test]
async fn me_returns_profile_with_success_envelope() {
    let ctx = TestContext::new().await;

    let (status, payload) =
        send_json(&ctx.app, Method::GET, "/api/v1/me", Some(&ctx.token), None)
            .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(payload["data"]["id"], ctx.user_id.to_string());
    assert_eq!(payload["data"]["email"], "qa@example.com");
    assert_eq!(payload["data"]["display_name"], "qa-user");
    assert!(payload["meta"]["request_id"].is_string());

    ctx.cleanup().await;
}

#[tokio::test]
async fn requests_crud_lifecycle_and_audit_are_consistent() {
    let ctx = TestContext::new().await;

    let create_body = json!({
        "title": "VPN access",
        "description": "Need VPN access for on-call",
        "category": "IT",
        "priority": "high"
    });

    let (create_status, create_payload) = send_json(
        &ctx.app,
        Method::POST,
        "/api/v1/requests",
        Some(&ctx.token),
        Some(create_body),
    )
    .await;

    assert_eq!(create_status, StatusCode::CREATED);
    let request_id = create_payload["data"]["id"]
        .as_str()
        .expect("request id should be present")
        .to_string();
    assert_eq!(create_payload["data"]["status"], "open");

    let path = format!("/api/v1/requests/{request_id}");
    let (get_status, get_payload) =
        send_json(&ctx.app, Method::GET, &path, Some(&ctx.token), None).await;
    assert_eq!(get_status, StatusCode::OK);
    assert_eq!(get_payload["data"]["title"], "VPN access");

    let (patch_status, patch_payload) = send_json(
        &ctx.app,
        Method::PATCH,
        &path,
        Some(&ctx.token),
        Some(json!({
            "status": "in_progress",
            "priority": "medium"
        })),
    )
    .await;
    assert_eq!(patch_status, StatusCode::OK);
    assert_eq!(patch_payload["data"]["status"], "in_progress");
    assert_eq!(patch_payload["data"]["priority"], "medium");

    let (audit_status, audit_payload) = send_json(
        &ctx.app,
        Method::GET,
        &format!("/api/v1/requests/{request_id}/audit"),
        Some(&ctx.token),
        None,
    )
    .await;
    assert_eq!(audit_status, StatusCode::OK);
    assert!(
        audit_payload["data"]
            .as_array()
            .expect("audit entries should be array")
            .len()
            >= 2
    );

    let (delete_status, delete_payload) =
        send_json(&ctx.app, Method::DELETE, &path, Some(&ctx.token), None)
            .await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);
    assert_eq!(delete_payload, json!({}));

    let (missing_status, missing_payload) =
        send_json(&ctx.app, Method::GET, &path, Some(&ctx.token), None).await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
    assert_eq!(missing_payload["error"]["code"], "NOT_FOUND");

    ctx.cleanup().await;
}

#[tokio::test]
async fn requests_list_supports_pagination_and_filters() {
    let ctx = TestContext::new().await;

    for (title, category, priority) in [
        ("Access HR drive", "HR", "low"),
        ("New laptop", "IT", "high"),
        ("Payroll issue", "Admin", "medium"),
    ] {
        let (status, _) = send_json(
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
        .await;
        assert_eq!(status, StatusCode::CREATED);
    }

    let (status, payload) = send_json(
        &ctx.app,
        Method::GET,
        "/api/v1/requests?limit=2&page=1&category=IT",
        Some(&ctx.token),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(payload["meta"]["page"], 1);
    assert_eq!(payload["meta"]["limit"], 2);
    assert_eq!(payload["meta"]["total"], 1);
    assert_eq!(
        payload["data"]
            .as_array()
            .expect("data should be array")
            .len(),
        1
    );

    ctx.cleanup().await;
}

#[tokio::test]
async fn request_validation_errors_use_error_envelope() {
    let ctx = TestContext::new().await;

    let (status, payload) = send_json(
        &ctx.app,
        Method::POST,
        "/api/v1/requests",
        Some(&ctx.token),
        Some(json!({
            "title": "",
            "description": "bad",
            "category": "Unknown",
            "priority": "urgent"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["error"]["code"], "VALIDATION_ERROR");
    assert!(payload["error"]["details"].is_array());

    ctx.cleanup().await;
}

fn build_token(user_id: Uuid) -> String {
    encode(
        &Header::default(),
        &TestClaims {
            sub: user_id.to_string(),
            email: "qa@example.com".to_string(),
            aud: "authenticated".to_string(),
            iss: TEST_JWT_ISSUER.to_string(),
            exp: 9_999_999_999,
        },
        &EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .expect("token should encode")
}

async fn send_json(
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

async fn bootstrap_supabase_compat(pool: &PgPool) {
    pool.execute(
        "CREATE SCHEMA IF NOT EXISTS auth;
         CREATE TABLE IF NOT EXISTS auth.users (
           id UUID PRIMARY KEY,
           email TEXT,
           raw_user_meta_data JSONB NOT NULL DEFAULT '{}'::jsonb
         );
         CREATE OR REPLACE FUNCTION auth.uid()
         RETURNS UUID
         LANGUAGE SQL
         STABLE
         AS $$
           SELECT NULLIF(current_setting('request.jwt.claim.sub', true), '')::uuid
         $$;",
    )
    .await
    .expect("supabase compatibility objects should be created");
}
