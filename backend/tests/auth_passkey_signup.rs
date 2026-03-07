mod support;

use axum::http::{Method, StatusCode};
use serde_json::json;

use support::{TestContext, send_json};

#[tokio::test]
async fn passkey_signup_start_supports_unauthenticated_new_email() {
    let ctx = TestContext::new().await;

    let (status, payload) = send_json(
        &ctx.app,
        Method::POST,
        "/api/v1/auth/passkeys/signup/start",
        None,
        Some(json!({
            "email": "passkey-only@example.com",
            "display_name": "Passkey Only User",
            "nickname": "Primary Key"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(payload["data"]["challenge_id"].is_string());
    assert!(payload["data"]["options"].is_object());

    ctx.cleanup().await;
}

#[tokio::test]
async fn passkey_signup_start_rejects_existing_email() {
    let ctx = TestContext::new().await;

    let (status, payload) = send_json(
        &ctx.app,
        Method::POST,
        "/api/v1/auth/passkeys/signup/start",
        None,
        Some(json!({
            "email": "qa@example.com",
            "display_name": "Already Exists"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["error"]["code"], "VALIDATION_ERROR");
    assert!(
        payload["error"]["details"][0]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("already exists")
    );

    ctx.cleanup().await;
}

#[tokio::test]
async fn passkey_login_start_requires_existing_passkeys() {
    let ctx = TestContext::new().await;

    let (status, payload) = send_json(
        &ctx.app,
        Method::POST,
        "/api/v1/auth/passkeys/login/start",
        None,
        Some(json!({})),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(payload["error"]["code"], "UNAUTHORIZED");
    assert_eq!(
        payload["error"]["message"],
        "unauthorized: no passkey registered for account"
    );

    ctx.cleanup().await;
}
