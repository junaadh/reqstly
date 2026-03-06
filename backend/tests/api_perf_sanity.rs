mod support;

use axum::http::Method;
use sqlx::Row;

use support::{TestContext, create_request, send_json};

#[tokio::test]
async fn list_limit_and_page_are_bounded() {
    let ctx = TestContext::new().await;

    for index in 0..12 {
        let (status, _) = create_request(
            &ctx,
            &format!("Perf request {index}"),
            "IT",
            "medium",
        )
        .await;
        assert_eq!(status.as_u16(), 201);
    }

    let (_status, payload) = send_json(
        &ctx.app,
        Method::GET,
        "/api/v1/requests?limit=999&page=0",
        Some(&ctx.token),
        None,
    )
    .await;

    assert_eq!(payload["meta"]["limit"], 100);
    assert_eq!(payload["meta"]["page"], 1);
    assert_eq!(payload["meta"]["total"], 12);
    assert_eq!(payload["meta"]["total_pages"], 1);

    ctx.cleanup().await;
}

#[tokio::test]
async fn default_sort_is_created_at_desc() {
    let ctx = TestContext::new().await;

    let mut created_ids = Vec::new();
    for title in ["First", "Second", "Third"] {
        let (status, payload) =
            create_request(&ctx, title, "IT", "medium").await;
        assert_eq!(status.as_u16(), 201);
        let id = payload["data"]["id"]
            .as_str()
            .expect("request id should be present")
            .to_string();
        created_ids.push(id);
    }

    let (_status, payload) = send_json(
        &ctx.app,
        Method::GET,
        "/api/v1/requests",
        Some(&ctx.token),
        None,
    )
    .await;

    let items = payload["data"]
        .as_array()
        .expect("data array should be present");
    assert!(!items.is_empty(), "list should return created items");

    let first_id = items[0]["id"]
        .as_str()
        .expect("first item id should be string");
    let last_created_id = created_ids.last().expect("at least one id");
    assert_eq!(
        first_id, last_created_id,
        "default list ordering should return newest request first"
    );

    ctx.cleanup().await;
}

#[tokio::test]
async fn hot_path_indexes_exist() {
    let ctx = TestContext::new().await;

    let index_rows = sqlx::query(
        "SELECT indexname
         FROM pg_indexes
         WHERE schemaname = 'app'
         ORDER BY indexname",
    )
    .fetch_all(&ctx.pool)
    .await
    .expect("index lookup should succeed");

    let indexes: std::collections::BTreeSet<String> = index_rows
        .iter()
        .map(|row| row.get::<String, _>("indexname"))
        .collect();

    for expected in [
        "idx_requests_owner_user_created_at",
        "idx_requests_owner_user_updated_at",
        "idx_requests_owner_user_status_created_at",
        "idx_requests_owner_user_category_created_at",
        "idx_requests_owner_user_priority_created_at",
        "idx_request_audit_logs_request_id_created_at",
        "idx_request_participants_user_request",
    ] {
        assert!(
            indexes.contains(expected),
            "expected index missing: {expected}"
        );
    }

    ctx.cleanup().await;
}
