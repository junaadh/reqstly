use std::{sync::Arc, time::Duration};

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
use tokio::{sync::mpsc, time::timeout};
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
    realtime_hub: reqstly_backend::realtime::RealtimeHub,
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

        let realtime_hub = reqstly_backend::realtime::RealtimeHub::new();
        let token = build_token(user_id);
        let app = build_app(
            AppState {
                db: pool.clone(),
                jwt_secret: TEST_JWT_SECRET.to_string(),
                jwt_issuer: TEST_JWT_ISSUER.to_string(),
                realtime_hub: realtime_hub.clone(),
                ws_allowed_origins: vec!["*".to_string()],
            },
            "*",
        )
        .expect("router should build");

        Self {
            app,
            pool,
            realtime_hub,
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
async fn patch_me_updates_display_name_and_returns_updated_profile() {
    let ctx = TestContext::new().await;

    let (patch_status, patch_payload) = send_json(
        &ctx.app,
        Method::PATCH,
        "/api/v1/me",
        Some(&ctx.token),
        Some(json!({
            "display_name": "Reqstly QA Lead"
        })),
    )
    .await;

    assert_eq!(patch_status, StatusCode::OK);
    assert_eq!(patch_payload["data"]["id"], ctx.user_id.to_string());
    assert_eq!(patch_payload["data"]["display_name"], "Reqstly QA Lead");
    assert!(patch_payload["meta"]["request_id"].is_string());

    let (get_status, get_payload) =
        send_json(&ctx.app, Method::GET, "/api/v1/me", Some(&ctx.token), None)
            .await;
    assert_eq!(get_status, StatusCode::OK);
    assert_eq!(get_payload["data"]["display_name"], "Reqstly QA Lead");

    ctx.cleanup().await;
}

#[tokio::test]
async fn patch_me_emits_profile_patch_to_same_user_connections() {
    let ctx = TestContext::new().await;

    let (_conn_a, mut user_rx_a) = ctx.realtime_hub.register(ctx.user_id).await;
    let (_conn_b, mut user_rx_b) = ctx.realtime_hub.register(ctx.user_id).await;

    let other_user_id = Uuid::new_v4();
    let (_other_conn, mut other_rx) =
        ctx.realtime_hub.register(other_user_id).await;

    let (status, payload) = send_json(
        &ctx.app,
        Method::PATCH,
        "/api/v1/me",
        Some(&ctx.token),
        Some(json!({
            "display_name": "Realtime Synced Name"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(payload["data"]["display_name"], "Realtime Synced Name");

    let first_event = recv_realtime_event(&mut user_rx_a).await;
    assert_eq!(first_event["type"], "profile.patch");
    assert_eq!(
        first_event["payload"]["user"]["id"],
        ctx.user_id.to_string()
    );
    assert_eq!(
        first_event["payload"]["user"]["display_name"],
        "Realtime Synced Name"
    );
    assert_eq!(
        first_event["payload"]["changed_fields"],
        json!(["display_name"])
    );

    let second_event = recv_realtime_event(&mut user_rx_b).await;
    assert_eq!(second_event["type"], "profile.patch");
    assert_eq!(
        second_event["payload"]["user"]["display_name"],
        "Realtime Synced Name"
    );

    assert_no_realtime_event(&mut other_rx).await;

    ctx.cleanup().await;
}

#[tokio::test]
async fn patch_me_validates_display_name() {
    let ctx = TestContext::new().await;

    let (status, payload) = send_json(
        &ctx.app,
        Method::PATCH,
        "/api/v1/me",
        Some(&ctx.token),
        Some(json!({
            "display_name": "   "
        })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["error"]["code"], "VALIDATION_ERROR");
    assert_eq!(payload["error"]["details"][0]["field"], "display_name");

    ctx.cleanup().await;
}

#[tokio::test]
async fn preferences_default_and_patch_persist() {
    let ctx = TestContext::new().await;

    let (get_status, get_payload) = send_json(
        &ctx.app,
        Method::GET,
        "/api/v1/preferences",
        Some(&ctx.token),
        None,
    )
    .await;

    assert_eq!(get_status, StatusCode::OK);
    assert_eq!(get_payload["data"]["email_digest"], true);
    assert_eq!(get_payload["data"]["browser_alerts"], true);
    assert_eq!(get_payload["data"]["default_page_size"], 20);

    let (patch_status, patch_payload) = send_json(
        &ctx.app,
        Method::PATCH,
        "/api/v1/preferences",
        Some(&ctx.token),
        Some(json!({
            "email_digest": false,
            "browser_alerts": false,
            "default_page_size": 50
        })),
    )
    .await;

    assert_eq!(patch_status, StatusCode::OK);
    assert_eq!(patch_payload["data"]["email_digest"], false);
    assert_eq!(patch_payload["data"]["browser_alerts"], false);
    assert_eq!(patch_payload["data"]["default_page_size"], 50);

    let (get_again_status, get_again_payload) = send_json(
        &ctx.app,
        Method::GET,
        "/api/v1/preferences",
        Some(&ctx.token),
        None,
    )
    .await;

    assert_eq!(get_again_status, StatusCode::OK);
    assert_eq!(get_again_payload["data"]["email_digest"], false);
    assert_eq!(get_again_payload["data"]["browser_alerts"], false);
    assert_eq!(get_again_payload["data"]["default_page_size"], 50);

    ctx.cleanup().await;
}

#[tokio::test]
async fn preferences_patch_validates_default_page_size() {
    let ctx = TestContext::new().await;

    let (status, payload) = send_json(
        &ctx.app,
        Method::PATCH,
        "/api/v1/preferences",
        Some(&ctx.token),
        Some(json!({
            "default_page_size": 33
        })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["error"]["code"], "VALIDATION_ERROR");
    assert_eq!(payload["error"]["details"][0]["field"], "default_page_size");

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
    assert_eq!(
        create_payload["data"]["assignee_user_id"],
        ctx.user_id.to_string()
    );
    assert_eq!(create_payload["data"]["assignee_email"], "qa@example.com");

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
    assert_eq!(audit_payload["data"][0]["actor_email"], "qa@example.com");

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
async fn requests_list_supports_tokenized_search_query() {
    let ctx = TestContext::new().await;

    let teammate_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO auth.users (id, email, raw_user_meta_data)
         VALUES ($1, $2, $3)",
    )
    .bind(teammate_id)
    .bind("teammate@example.com")
    .bind(json!({ "display_name": "Teammate User" }))
    .execute(&ctx.pool)
    .await
    .expect("teammate should insert");

    for (title, description, category, priority, assignee_email) in [
        (
            "VPN access setup",
            "Need access to corporate VPN",
            "IT",
            "high",
            Some("teammate@example.com"),
        ),
        (
            "Access card replacement",
            "Office access badge damaged",
            "Ops",
            "medium",
            None,
        ),
        (
            "Payroll follow-up",
            "Reimbursement mismatch",
            "Admin",
            "low",
            None,
        ),
    ] {
        let (status, _) = send_json(
            &ctx.app,
            Method::POST,
            "/api/v1/requests",
            Some(&ctx.token),
            Some(json!({
                "title": title,
                "description": description,
                "category": category,
                "priority": priority,
                "assignee_email": assignee_email
            })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
    }

    let (search_status, search_payload) = send_json(
        &ctx.app,
        Method::GET,
        "/api/v1/requests?limit=1&page=1&q=access",
        Some(&ctx.token),
        None,
    )
    .await;

    assert_eq!(search_status, StatusCode::OK);
    assert_eq!(search_payload["meta"]["total"], 2);
    assert_eq!(search_payload["meta"]["limit"], 1);
    assert_eq!(search_payload["meta"]["page"], 1);
    assert_eq!(
        search_payload["data"]
            .as_array()
            .expect("search data should be array")
            .len(),
        1
    );

    let (tokenized_status, tokenized_payload) = send_json(
        &ctx.app,
        Method::GET,
        "/api/v1/requests?q=vpn%20corporate",
        Some(&ctx.token),
        None,
    )
    .await;

    assert_eq!(tokenized_status, StatusCode::OK);
    assert_eq!(tokenized_payload["meta"]["total"], 1);
    assert_eq!(tokenized_payload["data"][0]["title"], "VPN access setup");
    assert_eq!(
        tokenized_payload["data"][0]["assignee_email"],
        "teammate@example.com"
    );

    let (filtered_status, filtered_payload) = send_json(
        &ctx.app,
        Method::GET,
        "/api/v1/requests?category=Admin&q=access",
        Some(&ctx.token),
        None,
    )
    .await;

    assert_eq!(filtered_status, StatusCode::OK);
    assert_eq!(filtered_payload["meta"]["total"], 0);
    assert_eq!(
        filtered_payload["data"]
            .as_array()
            .expect("filtered data should be array")
            .len(),
        0
    );

    ctx.cleanup().await;
}

#[tokio::test]
async fn request_assignment_and_domain_suggestions_work() {
    let ctx = TestContext::new().await;

    let teammate_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO auth.users (id, email, raw_user_meta_data)
         VALUES ($1, $2, $3)",
    )
    .bind(teammate_id)
    .bind("teammate@example.com")
    .bind(json!({ "display_name": "Teammate User" }))
    .execute(&ctx.pool)
    .await
    .expect("teammate should insert");

    let collaborator_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO auth.users (id, email, raw_user_meta_data)
         VALUES ($1, $2, $3)",
    )
    .bind(collaborator_id)
    .bind("collab@example.com")
    .bind(json!({ "display_name": "Collab Ops" }))
    .execute(&ctx.pool)
    .await
    .expect("collaborator should insert");

    let external_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO auth.users (id, email, raw_user_meta_data)
         VALUES ($1, $2, $3)",
    )
    .bind(external_id)
    .bind("external@other.com")
    .bind(json!({ "display_name": "External User" }))
    .execute(&ctx.pool)
    .await
    .expect("external user should insert");

    for assignee_email in [
        "teammate@example.com",
        "teammate@example.com",
        "collab@example.com",
    ] {
        let (seed_status, _) = send_json(
            &ctx.app,
            Method::POST,
            "/api/v1/requests",
            Some(&ctx.token),
            Some(json!({
                "title": format!("seed-{assignee_email}"),
                "description": null,
                "category": "IT",
                "priority": "low",
                "assignee_email": assignee_email
            })),
        )
        .await;
        assert_eq!(seed_status, StatusCode::CREATED);
    }

    let (suggest_status, suggest_payload) = send_json(
        &ctx.app,
        Method::GET,
        "/api/v1/assignees/suggestions",
        Some(&ctx.token),
        None,
    )
    .await;
    assert_eq!(suggest_status, StatusCode::OK);
    let suggestion_emails = suggest_payload["data"]
        .as_array()
        .expect("suggestions should be an array")
        .iter()
        .filter_map(|item| item["email"].as_str())
        .collect::<Vec<_>>();
    assert!(!suggestion_emails.contains(&"qa@example.com"));
    assert!(suggestion_emails.contains(&"teammate@example.com"));
    assert!(suggestion_emails.contains(&"collab@example.com"));
    assert!(!suggestion_emails.contains(&"external@other.com"));
    assert_eq!(suggest_payload["data"][0]["email"], "teammate@example.com");
    assert_eq!(suggest_payload["data"][0]["assignment_count"], 2);

    let (search_status, search_payload) = send_json(
        &ctx.app,
        Method::GET,
        "/api/v1/assignees/suggestions?q=team",
        Some(&ctx.token),
        None,
    )
    .await;
    assert_eq!(search_status, StatusCode::OK);
    assert_eq!(search_payload["data"][0]["email"], "teammate@example.com");
    assert_eq!(
        search_payload["data"]
            .as_array()
            .expect("search results should be array")
            .len(),
        1
    );

    let (create_status, create_payload) = send_json(
        &ctx.app,
        Method::POST,
        "/api/v1/requests",
        Some(&ctx.token),
        Some(json!({
            "title": "Assign request",
            "description": "Needs teammate ownership",
            "category": "IT",
            "priority": "medium",
            "assignee_email": "teammate@example.com"
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    assert_eq!(
        create_payload["data"]["assignee_email"],
        "teammate@example.com"
    );
    assert_eq!(
        create_payload["data"]["assignee_display_name"],
        "Teammate User"
    );
    assert_eq!(
        create_payload["data"]["assignee_user_id"],
        teammate_id.to_string()
    );
    let request_id = create_payload["data"]["id"]
        .as_str()
        .expect("request id should exist")
        .to_string();
    let teammate_token =
        build_token_with_email(teammate_id, "teammate@example.com");

    let (teammate_list_status, teammate_list_payload) = send_json(
        &ctx.app,
        Method::GET,
        "/api/v1/requests",
        Some(&teammate_token),
        None,
    )
    .await;
    assert_eq!(teammate_list_status, StatusCode::OK);
    assert!(
        teammate_list_payload["data"]
            .as_array()
            .expect("teammate list should be array")
            .iter()
            .any(|item| item["id"] == request_id),
        "teammate should see request while assigned"
    );

    let (teammate_get_status, teammate_get_payload) = send_json(
        &ctx.app,
        Method::GET,
        &format!("/api/v1/requests/{request_id}"),
        Some(&teammate_token),
        None,
    )
    .await;
    assert_eq!(teammate_get_status, StatusCode::OK);
    assert_eq!(teammate_get_payload["data"]["id"], request_id);

    let (teammate_patch_status, teammate_patch_payload) = send_json(
        &ctx.app,
        Method::PATCH,
        &format!("/api/v1/requests/{request_id}"),
        Some(&teammate_token),
        Some(json!({
            "status": "in_progress"
        })),
    )
    .await;
    assert_eq!(teammate_patch_status, StatusCode::OK);
    assert_eq!(teammate_patch_payload["data"]["status"], "in_progress");

    let (patch_status, patch_payload) = send_json(
        &ctx.app,
        Method::PATCH,
        &format!("/api/v1/requests/{request_id}"),
        Some(&ctx.token),
        Some(json!({
            "assignee_email": ""
        })),
    )
    .await;
    assert_eq!(patch_status, StatusCode::OK);
    assert!(patch_payload["data"]["assignee_user_id"].is_null());
    assert!(patch_payload["data"]["assignee_email"].is_null());
    assert!(patch_payload["data"]["assignee_display_name"].is_null());

    let (teammate_post_unassign_status, teammate_post_unassign_payload) =
        send_json(
            &ctx.app,
            Method::GET,
            "/api/v1/requests",
            Some(&teammate_token),
            None,
        )
        .await;
    assert_eq!(teammate_post_unassign_status, StatusCode::OK);
    assert!(
        teammate_post_unassign_payload["data"]
            .as_array()
            .expect("teammate list should be array")
            .iter()
            .any(|item| item["id"] == request_id),
        "teammate should continue seeing request once their email exists in audit log"
    );

    let (teammate_audit_status, teammate_audit_payload) = send_json(
        &ctx.app,
        Method::GET,
        &format!("/api/v1/requests/{request_id}/audit"),
        Some(&teammate_token),
        None,
    )
    .await;
    assert_eq!(teammate_audit_status, StatusCode::OK);
    assert!(
        teammate_audit_payload["data"]
            .as_array()
            .expect("audit list should be array")
            .len()
            >= 2
    );

    let (invalid_status, invalid_payload) = send_json(
        &ctx.app,
        Method::PATCH,
        &format!("/api/v1/requests/{request_id}"),
        Some(&ctx.token),
        Some(json!({
            "assignee_email": "missing@example.com"
        })),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_payload["error"]["code"], "VALIDATION_ERROR");
    assert_eq!(
        invalid_payload["error"]["details"][0]["field"],
        "assignee_email"
    );

    ctx.cleanup().await;
}

#[tokio::test]
async fn realtime_fanout_for_assign_status_and_delete_is_consistent() {
    let ctx = TestContext::new().await;

    let teammate_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO auth.users (id, email, raw_user_meta_data)
         VALUES ($1, $2, $3)",
    )
    .bind(teammate_id)
    .bind("teammate@example.com")
    .bind(json!({ "display_name": "Teammate User" }))
    .execute(&ctx.pool)
    .await
    .expect("teammate should insert");

    let observer_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO auth.users (id, email, raw_user_meta_data)
         VALUES ($1, $2, $3)",
    )
    .bind(observer_id)
    .bind("observer@example.com")
    .bind(json!({ "display_name": "Observer User" }))
    .execute(&ctx.pool)
    .await
    .expect("observer should insert");

    let teammate_token =
        build_token_with_email(teammate_id, "teammate@example.com");

    let (_owner_connection_id, mut owner_rx) =
        ctx.realtime_hub.register(ctx.user_id).await;
    let (_teammate_connection_id, mut teammate_rx) =
        ctx.realtime_hub.register(teammate_id).await;
    let (_observer_connection_id, mut observer_rx) =
        ctx.realtime_hub.register(observer_id).await;

    let (create_status, create_payload) = send_json(
        &ctx.app,
        Method::POST,
        "/api/v1/requests",
        Some(&ctx.token),
        Some(json!({
            "title": "Realtime fanout request",
            "description": "Validate realtime event fanout",
            "category": "IT",
            "priority": "medium"
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    let request_id = create_payload["data"]["id"]
        .as_str()
        .expect("request id should be present")
        .to_string();

    let owner_create_event = recv_realtime_event(&mut owner_rx).await;
    assert_eq!(owner_create_event["type"], "request.created");
    assert_eq!(owner_create_event["payload"]["request"]["id"], request_id);
    let owner_create_audit_event = recv_realtime_event(&mut owner_rx).await;
    assert_eq!(owner_create_audit_event["type"], "audit.append");
    assert_eq!(
        owner_create_audit_event["payload"]["audit"]["request_id"],
        request_id
    );
    assert_no_realtime_event(&mut teammate_rx).await;
    assert_no_realtime_event(&mut observer_rx).await;

    let (assign_status, assign_payload) = send_json(
        &ctx.app,
        Method::PATCH,
        &format!("/api/v1/requests/{request_id}"),
        Some(&ctx.token),
        Some(json!({
            "assignee_email": "teammate@example.com"
        })),
    )
    .await;
    assert_eq!(assign_status, StatusCode::OK);
    assert_eq!(
        assign_payload["data"]["assignee_email"],
        "teammate@example.com"
    );

    let owner_assign_patch = recv_realtime_event(&mut owner_rx).await;
    assert_eq!(owner_assign_patch["type"], "request.patch");
    assert_eq!(
        owner_assign_patch["payload"]["request"]["assignee_email"],
        "teammate@example.com"
    );
    let owner_assign_audit = recv_realtime_event(&mut owner_rx).await;
    assert_eq!(owner_assign_audit["type"], "audit.append");

    let teammate_assign_created = recv_realtime_event(&mut teammate_rx).await;
    assert_eq!(teammate_assign_created["type"], "request.created");
    assert_eq!(
        teammate_assign_created["payload"]["request"]["id"],
        request_id
    );
    let teammate_assign_audit = recv_realtime_event(&mut teammate_rx).await;
    assert_eq!(teammate_assign_audit["type"], "audit.append");
    assert_no_realtime_event(&mut observer_rx).await;

    let (status_patch_status, status_patch_payload) = send_json(
        &ctx.app,
        Method::PATCH,
        &format!("/api/v1/requests/{request_id}"),
        Some(&teammate_token),
        Some(json!({
            "status": "in_progress"
        })),
    )
    .await;
    assert_eq!(status_patch_status, StatusCode::OK);
    assert_eq!(status_patch_payload["data"]["status"], "in_progress");

    let owner_status_patch = recv_realtime_event(&mut owner_rx).await;
    assert_eq!(owner_status_patch["type"], "request.patch");
    assert_eq!(
        owner_status_patch["payload"]["request"]["status"],
        "in_progress"
    );
    assert_eq!(owner_status_patch["payload"]["previous_status"], "open");
    let owner_status_audit = recv_realtime_event(&mut owner_rx).await;
    assert_eq!(owner_status_audit["type"], "audit.append");

    let teammate_status_patch = recv_realtime_event(&mut teammate_rx).await;
    assert_eq!(teammate_status_patch["type"], "request.patch");
    assert_eq!(
        teammate_status_patch["payload"]["request"]["status"],
        "in_progress"
    );
    let teammate_status_audit = recv_realtime_event(&mut teammate_rx).await;
    assert_eq!(teammate_status_audit["type"], "audit.append");
    assert_no_realtime_event(&mut observer_rx).await;

    let (delete_status, delete_payload) = send_json(
        &ctx.app,
        Method::DELETE,
        &format!("/api/v1/requests/{request_id}"),
        Some(&ctx.token),
        None,
    )
    .await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);
    assert_eq!(delete_payload, json!({}));

    let owner_delete_audit = recv_realtime_event(&mut owner_rx).await;
    assert_eq!(owner_delete_audit["type"], "audit.append");
    let owner_delete_event = recv_realtime_event(&mut owner_rx).await;
    assert_eq!(owner_delete_event["type"], "request.deleted");
    assert_eq!(owner_delete_event["payload"]["id"], request_id);
    assert_eq!(owner_delete_event["payload"]["status"], "in_progress");

    let teammate_delete_audit = recv_realtime_event(&mut teammate_rx).await;
    assert_eq!(teammate_delete_audit["type"], "audit.append");
    let teammate_delete_event = recv_realtime_event(&mut teammate_rx).await;
    assert_eq!(teammate_delete_event["type"], "request.deleted");
    assert_eq!(teammate_delete_event["payload"]["id"], request_id);
    assert_eq!(teammate_delete_event["payload"]["status"], "in_progress");
    assert_no_realtime_event(&mut observer_rx).await;

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

async fn recv_realtime_event(receiver: &mut mpsc::Receiver<Arc<str>>) -> Value {
    let raw = timeout(Duration::from_secs(2), receiver.recv())
        .await
        .expect("realtime event should arrive within timeout")
        .expect("realtime receiver should return a message");

    serde_json::from_str(raw.as_ref()).expect("realtime message should be json")
}

async fn assert_no_realtime_event(receiver: &mut mpsc::Receiver<Arc<str>>) {
    let maybe_event =
        timeout(Duration::from_millis(200), receiver.recv()).await;
    assert!(maybe_event.is_err(), "unexpected realtime event received");
}

fn build_token(user_id: Uuid) -> String {
    build_token_with_email(user_id, "qa@example.com")
}

fn build_token_with_email(user_id: Uuid, email: &str) -> String {
    encode(
        &Header::default(),
        &TestClaims {
            sub: user_id.to_string(),
            email: email.to_string(),
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
