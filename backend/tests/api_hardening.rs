mod support;

use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::Error as SqlxError;
use tower::util::ServiceExt;
use uuid::Uuid;

use support::{
    TestClaims, TestContext, build_token, build_token_with_claims,
    create_request, insert_auth_user, insert_ws_token_issuance, send_json,
};

async fn assert_me_unauthorized_for_token(
    ctx: &TestContext,
    token: String,
) -> Value {
    let (status, payload) =
        send_json(&ctx.app, Method::GET, "/api/v1/me", Some(&token), None)
            .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(payload["error"]["code"], "UNAUTHORIZED");
    assert!(payload["meta"]["request_id"].is_string());
    payload
}

#[tokio::test]
async fn auth_rejects_expired_token() {
    let ctx = TestContext::new().await;

    let token = build_token_with_claims(
        TestClaims {
            sub: ctx.user_id.to_string(),
            email: "qa@example.com".to_string(),
            aud: "ws".to_string(),
            iss: support::TEST_WS_TOKEN_ISSUER.to_string(),
            exp: 1,
        },
        support::TEST_WS_TOKEN_SECRET,
    );

    let _ = assert_me_unauthorized_for_token(&ctx, token).await;
    ctx.cleanup().await;
}

#[tokio::test]
async fn auth_rejects_wrong_issuer_and_audience() {
    let ctx = TestContext::new().await;

    let wrong_issuer_token = build_token_with_claims(
        TestClaims {
            sub: ctx.user_id.to_string(),
            email: "qa@example.com".to_string(),
            aud: "ws".to_string(),
            iss: "https://issuer.invalid/auth/v1".to_string(),
            exp: 9_999_999_999,
        },
        support::TEST_WS_TOKEN_SECRET,
    );
    let _ = assert_me_unauthorized_for_token(&ctx, wrong_issuer_token).await;

    let wrong_audience_token = build_token_with_claims(
        TestClaims {
            sub: ctx.user_id.to_string(),
            email: "qa@example.com".to_string(),
            aud: "anon".to_string(),
            iss: support::TEST_WS_TOKEN_ISSUER.to_string(),
            exp: 9_999_999_999,
        },
        support::TEST_WS_TOKEN_SECRET,
    );
    let _ = assert_me_unauthorized_for_token(&ctx, wrong_audience_token).await;

    ctx.cleanup().await;
}

#[tokio::test]
async fn auth_rejects_malformed_bearer_header() {
    let ctx = TestContext::new().await;
    let token = build_token(ctx.user_id);

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/me")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Token {token}"))
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should execute");

    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("response body should collect")
        .to_bytes();
    let payload: Value =
        serde_json::from_slice(&body).expect("response should be json");

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(payload["error"]["code"], "UNAUTHORIZED");
    assert!(payload["meta"]["request_id"].is_string());

    ctx.cleanup().await;
}

#[tokio::test]
async fn auth_rejects_unissued_bearer_token() {
    let ctx = TestContext::new().await;
    let unissued_user_id = Uuid::new_v4();
    insert_auth_user(
        &ctx.pool,
        unissued_user_id,
        "unissued@example.com",
        "unissued-user",
    )
    .await;

    let token = build_token(unissued_user_id);
    let _ = assert_me_unauthorized_for_token(&ctx, token).await;

    ctx.cleanup().await;
}

#[tokio::test]
async fn auth_revoke_all_sessions_revokes_active_ws_tokens() {
    let ctx = TestContext::new().await;

    let (revoke_status, revoke_payload) = send_json(
        &ctx.app,
        Method::POST,
        "/api/v1/auth/sessions/revoke",
        Some(&ctx.token),
        Some(json!({})),
    )
    .await;
    assert_eq!(revoke_status, StatusCode::OK);
    assert_eq!(revoke_payload["data"]["ok"], true);

    let (me_status, me_payload) =
        send_json(&ctx.app, Method::GET, "/api/v1/me", Some(&ctx.token), None)
            .await;
    assert_eq!(me_status, StatusCode::UNAUTHORIZED);
    assert_eq!(me_payload["error"]["code"], "UNAUTHORIZED");

    ctx.cleanup().await;
}

#[tokio::test]
async fn ownership_isolation_blocks_cross_user_access() {
    let ctx = TestContext::new().await;

    let (create_status, create_payload) =
        create_request(&ctx, "Owner request", "IT", "medium").await;
    assert_eq!(create_status, StatusCode::CREATED);
    let request_id = create_payload["data"]["id"]
        .as_str()
        .expect("request id should be present")
        .to_string();

    let other_user_id = Uuid::new_v4();
    insert_auth_user(
        &ctx.pool,
        other_user_id,
        "other@example.com",
        "other-user",
    )
    .await;
    let other_token = build_token(other_user_id);
    insert_ws_token_issuance(&ctx.pool, other_user_id, &other_token).await;

    let path = format!("/api/v1/requests/{request_id}");
    for (method, body) in [
        (Method::GET, None),
        (
            Method::PATCH,
            Some(json!({
                "priority": "high"
            })),
        ),
        (Method::DELETE, None),
    ] {
        let (status, payload) =
            send_json(&ctx.app, method, &path, Some(&other_token), body).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(payload["error"]["code"], "NOT_FOUND");
        assert!(payload["meta"]["request_id"].is_string());
    }

    let (audit_status, audit_payload) = send_json(
        &ctx.app,
        Method::GET,
        &format!("/api/v1/requests/{request_id}/audit"),
        Some(&other_token),
        None,
    )
    .await;
    assert_eq!(audit_status, StatusCode::NOT_FOUND);
    assert_eq!(audit_payload["error"]["code"], "NOT_FOUND");
    assert!(audit_payload["meta"]["request_id"].is_string());

    let (list_status, list_payload) = send_json(
        &ctx.app,
        Method::GET,
        "/api/v1/requests",
        Some(&other_token),
        None,
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    assert_eq!(list_payload["meta"]["total"], 0);
    assert_eq!(
        list_payload["data"]
            .as_array()
            .expect("data should be array")
            .len(),
        0
    );

    ctx.cleanup().await;
}

fn assert_database_constraint_error(err: SqlxError) {
    match err {
        SqlxError::Database(db_err) => {
            let message = db_err.message().to_lowercase();
            assert!(
                message.contains("constraint")
                    || message.contains("violates")
                    || message.contains("foreign key"),
                "unexpected database error message: {message}"
            );
        }
        other => panic!("expected database constraint error, got: {other:?}"),
    }
}

#[tokio::test]
async fn db_constraints_and_validation_paths_are_enforced() {
    let ctx = TestContext::new().await;

    let invalid_category_err = sqlx::query(
        "INSERT INTO app.requests (owner_user_id, title, category, status, priority)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(ctx.user_id)
    .bind("Invalid category row")
    .bind("Finance")
    .bind("open")
    .bind("low")
    .execute(&ctx.pool)
    .await
    .expect_err("db check constraint should reject invalid category");
    assert_database_constraint_error(invalid_category_err);

    let invalid_assignee_err = sqlx::query(
        "INSERT INTO app.requests (owner_user_id, title, category, status, priority, assignee_user_id)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(ctx.user_id)
    .bind("Invalid assignee row")
    .bind("IT")
    .bind("open")
    .bind("low")
    .bind(Uuid::new_v4())
    .execute(&ctx.pool)
    .await
    .expect_err("db fk should reject unknown assignee");
    assert_database_constraint_error(invalid_assignee_err);

    let (status, payload) = send_json(
        &ctx.app,
        Method::POST,
        "/api/v1/requests",
        Some(&ctx.token),
        Some(json!({
            "title": "x".repeat(256),
            "description": "too long title should fail validation",
            "category": "IT",
            "priority": "medium"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["error"]["code"], "VALIDATION_ERROR");
    assert!(payload["error"]["details"].is_array());

    ctx.cleanup().await;
}
