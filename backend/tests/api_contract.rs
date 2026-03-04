mod support;

use axum::http::{Method, StatusCode};
use serde_json::Value;
use uuid::Uuid;

use support::{TestContext, send_json};

fn assert_success_envelope(payload: &Value) {
    assert!(payload.get("data").is_some(), "missing data envelope");
    assert!(
        payload
            .get("meta")
            .and_then(|meta| meta.get("request_id"))
            .and_then(Value::as_str)
            .is_some(),
        "missing meta.request_id"
    );
}

fn assert_error_envelope(payload: &Value, expected_code: &str) {
    assert_eq!(payload["error"]["code"], expected_code);
    assert!(payload["error"]["message"].is_string());
    assert!(
        payload
            .get("meta")
            .and_then(|meta| meta.get("request_id"))
            .and_then(Value::as_str)
            .is_some(),
        "missing meta.request_id"
    );
}

#[tokio::test]
async fn health_contract_matches_envelope_and_payload() {
    let ctx = TestContext::new().await;

    let (root_status, root_payload) =
        send_json(&ctx.app, Method::GET, "/health", None, None).await;
    assert_eq!(root_status, StatusCode::OK);
    assert_success_envelope(&root_payload);
    assert_eq!(root_payload["data"]["status"], "ok");
    assert_eq!(root_payload["data"]["service"], "reqstly_backend");
    assert!(root_payload["data"]["version"].is_string());

    let (v1_status, v1_payload) =
        send_json(&ctx.app, Method::GET, "/api/v1/health", None, None).await;
    assert_eq!(v1_status, StatusCode::OK);
    assert_success_envelope(&v1_payload);
    assert_eq!(v1_payload["data"]["status"], "ok");
    assert_eq!(v1_payload["data"]["service"], "reqstly_backend");
    assert!(v1_payload["data"]["version"].is_string());

    ctx.cleanup().await;
}

#[tokio::test]
async fn unauthorized_and_not_found_errors_match_contract() {
    let ctx = TestContext::new().await;

    let (unauth_status, unauth_payload) =
        send_json(&ctx.app, Method::GET, "/api/v1/me", None, None).await;
    assert_eq!(unauth_status, StatusCode::UNAUTHORIZED);
    assert_error_envelope(&unauth_payload, "UNAUTHORIZED");

    let missing_id = Uuid::new_v4();
    let path = format!("/api/v1/requests/{missing_id}");
    let (missing_status, missing_payload) =
        send_json(&ctx.app, Method::GET, &path, Some(&ctx.token), None).await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
    assert_error_envelope(&missing_payload, "NOT_FOUND");

    ctx.cleanup().await;
}
