use std::collections::HashSet;

use axum::{
    Json, Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, PgPool};
use tokio::time::{self, Duration, Instant};
use tower_sessions::Session;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::{
    AppState,
    auth::{middleware, routes as auth_routes},
    error::{AppError, ErrorDetail},
    realtime::{self, ClientMessage, EventEnvelope},
    response,
};

#[derive(Debug, Serialize, FromRow)]
struct AuthUserRow {
    id: Uuid,
    email: String,
    display_name: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
struct RequestRow {
    id: Uuid,
    owner_user_id: Uuid,
    title: String,
    description: Option<String>,
    category: String,
    status: String,
    priority: String,
    assignee_user_id: Option<Uuid>,
    assignee_email: Option<String>,
    assignee_display_name: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
struct AuditLogRow {
    id: Uuid,
    request_id: Uuid,
    actor_user_id: Uuid,
    actor_email: String,
    action: String,
    old_value: serde_json::Value,
    new_value: serde_json::Value,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct WsAuthQuery {
    token: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
struct AssigneeSuggestionRow {
    id: Uuid,
    email: String,
    display_name: String,
    assignment_count: i64,
}

#[derive(Debug, Deserialize)]
struct ListRequestsQuery {
    status: Option<String>,
    category: Option<String>,
    priority: Option<String>,
    q: Option<String>,
    sort: Option<String>,
    page: Option<u64>,
    limit: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CreateRequestInput {
    title: String,
    description: Option<String>,
    category: String,
    priority: String,
    assignee_email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateRequestInput {
    title: Option<String>,
    description: Option<String>,
    category: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    assignee_email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AssigneeSuggestionsQuery {
    q: Option<String>,
    limit: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct UpdatePreferencesInput {
    email_digest: Option<bool>,
    browser_alerts: Option<bool>,
    default_page_size: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct UpdateMeInput {
    display_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct MeResponse {
    id: Uuid,
    email: String,
    display_name: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
struct PreferencesResponse {
    email_digest: bool,
    browser_alerts: bool,
    default_page_size: i32,
}

#[derive(Debug, Clone)]
struct PreferencesUpdate {
    email_digest: bool,
    browser_alerts: bool,
    default_page_size: i32,
}

#[derive(Debug, Serialize)]
struct RequestCreatedEventPayload {
    request: RequestRow,
}

#[derive(Debug, Serialize)]
struct RequestPatchEventPayload {
    request: RequestRow,
    changed_fields: Vec<String>,
    previous_status: String,
}

#[derive(Debug, Serialize)]
struct RequestDeletedEventPayload {
    id: Uuid,
    status: String,
}

#[derive(Debug, Serialize)]
struct AuditAppendEventPayload {
    audit: AuditLogRow,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(auth_routes::router())
        .route("/health", get(health))
        .route("/me", get(me).patch(update_me))
        .route(
            "/preferences",
            get(get_preferences).patch(update_preferences),
        )
        .route("/meta/enums", get(get_enums))
        .route("/assignees/suggestions", get(list_assignee_suggestions))
        .route("/requests", get(list_requests).post(create_request))
        .route(
            "/requests/:id",
            get(get_request)
                .patch(update_request)
                .delete(delete_request),
        )
        .route("/requests/:id/audit", get(get_request_audit))
}

pub async fn health(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await?;

    Ok(response::ok(
        StatusCode::OK,
        json!({
            "status": "ok",
            "service": "reqstly_backend",
            "version": "0.2.0"
        }),
    ))
}

pub async fn ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    Query(query): Query<WsAuthQuery>,
) -> Result<Response, AppError> {
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok());

    if !realtime::is_origin_allowed(origin, &state.ws_allowed_origins) {
        return Err(AppError::Unauthorized(
            "origin is not allowed for websocket".to_string(),
        ));
    }

    let trace_id = headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);

    let user_id =
        resolve_ws_user_id(&state, &session, &headers, query.token.as_deref())
            .await?;
    let hub = state.realtime_hub.clone();

    Ok(ws
        .max_frame_size(realtime::WS_MAX_MESSAGE_BYTES)
        .max_message_size(realtime::WS_MAX_MESSAGE_BYTES)
        .on_upgrade(move |socket| async move {
            handle_ws_connection(socket, hub, user_id, trace_id).await;
        }))
}

fn extract_ws_token(
    headers: &HeaderMap,
    query_token: Option<&str>,
) -> Result<String, AppError> {
    if headers.contains_key(header::AUTHORIZATION) {
        return middleware::extract_bearer_token(headers)
            .map(ToString::to_string);
    }

    let token = query_token
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .ok_or_else(|| {
            AppError::Unauthorized(
                "missing websocket authentication token".to_string(),
            )
        })?;

    Ok(token.to_string())
}

async fn resolve_ws_user_id(
    state: &AppState,
    session_handle: &Session,
    headers: &HeaderMap,
    query_token: Option<&str>,
) -> Result<Uuid, AppError> {
    let ws_token = extract_ws_token(headers, query_token).ok();

    if let Some(token) = ws_token {
        return middleware::verify_ws_token_with_state(state, &token).await;
    }

    let context =
        middleware::resolve_request_auth(state, session_handle, headers).await;
    match context {
        Ok(auth) => Ok(auth.user.id),
        Err(AppError::Unauthorized(_)) => Err(AppError::Unauthorized(
            "missing websocket authentication token".to_string(),
        )),
        Err(other) => Err(other),
    }
}

async fn handle_ws_connection(
    socket: WebSocket,
    hub: realtime::RealtimeHub,
    user_id: Uuid,
    trace_id: Option<String>,
) {
    let (connection_id, mut outbound) = hub.register(user_id).await;
    let (mut sender, mut receiver) = socket.split();

    let mut heartbeat = time::interval(Duration::from_secs(
        realtime::WS_HEARTBEAT_INTERVAL_SECS,
    ));
    heartbeat.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

    let mut idle_deadline =
        Instant::now() + Duration::from_secs(realtime::WS_IDLE_TIMEOUT_SECS);

    debug!(%user_id, %connection_id, ?trace_id, "websocket connected");

    loop {
        let sleep = time::sleep_until(idle_deadline);
        tokio::pin!(sleep);

        tokio::select! {
            _ = &mut sleep => {
                warn!(%user_id, %connection_id, "websocket idle timeout reached");
                break;
            }
            _ = heartbeat.tick() => {
                if sender.send(Message::Ping(Vec::new())).await.is_err() {
                    break;
                }
            }
            maybe_outbound = outbound.recv() => {
                let Some(text) = maybe_outbound else {
                    break;
                };

                if sender
                    .send(Message::Text(text.to_string()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            maybe_inbound = receiver.next() => {
                match maybe_inbound {
                    Some(Ok(Message::Text(text))) => {
                        idle_deadline = Instant::now()
                            + Duration::from_secs(realtime::WS_IDLE_TIMEOUT_SECS);
                        let sync_required =
                            handle_client_message(&text, user_id, connection_id);
                        if sync_required {
                            match EventEnvelope::new(
                                "sync.required",
                                None,
                                trace_id.clone(),
                                json!({}),
                            )
                            .encode()
                            {
                                Ok(message) => {
                                    if sender
                                        .send(Message::Text(message.to_string()))
                                        .await
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                                Err(err) => {
                                    warn!(
                                        %user_id,
                                        %connection_id,
                                        error = %err,
                                        "failed to encode sync.required event",
                                    );
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        idle_deadline = Instant::now()
                            + Duration::from_secs(realtime::WS_IDLE_TIMEOUT_SECS);
                        if sender.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        idle_deadline = Instant::now()
                            + Duration::from_secs(realtime::WS_IDLE_TIMEOUT_SECS);
                    }
                    Some(Ok(Message::Binary(_))) => {
                        warn!(%user_id, %connection_id, "websocket binary frame rejected");
                        break;
                    }
                    Some(Ok(Message::Close(_))) => {
                        break;
                    }
                    Some(Err(err)) => {
                        warn!(%user_id, %connection_id, error = %err, "websocket read error");
                        break;
                    }
                    None => {
                        break;
                    }
                }
            }
        }
    }

    hub.unregister(user_id, connection_id).await;
    let _ = sender.close().await;
    debug!(%user_id, %connection_id, "websocket disconnected");
}

fn handle_client_message(
    text: &str,
    user_id: Uuid,
    connection_id: Uuid,
) -> bool {
    if text.len() > realtime::WS_MAX_MESSAGE_BYTES {
        warn!(%user_id, %connection_id, "websocket message exceeded max size");
        return false;
    }

    match serde_json::from_str::<ClientMessage>(text) {
        Ok(message) => {
            let last_seen_ts = message.last_seen_ts();
            debug!(
                %user_id,
                %connection_id,
                last_seen_ts = ?last_seen_ts,
                "websocket client message received",
            );
            last_seen_ts.is_some()
        }
        Err(_) => {
            debug!(%user_id, %connection_id, "websocket client message ignored");
            false
        }
    }
}

async fn me(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let user = require_authenticated_user(&state, &session, &headers).await?;

    Ok(response::ok(
        StatusCode::OK,
        MeResponse {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
        },
    ))
}

async fn update_me(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    Json(input): Json<UpdateMeInput>,
) -> Result<impl IntoResponse, AppError> {
    let current_user =
        require_authenticated_user(&state, &session, &headers).await?;
    middleware::require_csrf_token(&state, &session, current_user.id, &headers)
        .await?;
    let display_name = normalize_display_name(input.display_name.as_deref())?;

    let user = sqlx::query_as::<_, AuthUserRow>(
        "UPDATE app.app_users
         SET display_name = $2,
             updated_at = NOW()
         WHERE id = $1
         RETURNING id, COALESCE(email, '') AS email, display_name",
    )
    .bind(current_user.id)
    .bind(display_name)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("auth user not found".to_string()))?;

    publish_event(
        &state,
        &[current_user.id],
        "profile.patch",
        None,
        json!({
            "user": {
                "id": user.id,
                "email": user.email,
                "display_name": user.display_name,
            },
            "changed_fields": ["display_name"],
        }),
    )
    .await;

    Ok(response::ok(
        StatusCode::OK,
        MeResponse {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
        },
    ))
}

async fn get_preferences(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let user = require_authenticated_user(&state, &session, &headers).await?;
    let preferences = fetch_or_create_preferences(&state.db, user.id).await?;

    Ok(response::ok(StatusCode::OK, preferences))
}

async fn update_preferences(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    Json(input): Json<UpdatePreferencesInput>,
) -> Result<impl IntoResponse, AppError> {
    let user = require_authenticated_user(&state, &session, &headers).await?;
    middleware::require_csrf_token(&state, &session, user.id, &headers).await?;
    let current = fetch_or_create_preferences(&state.db, user.id).await?;
    let update = normalize_preferences_update(input, &current)?;
    let updated = upsert_preferences(&state.db, user.id, &update).await?;

    Ok(response::ok(StatusCode::OK, updated))
}

async fn get_enums() -> impl IntoResponse {
    response::ok(
        StatusCode::OK,
        json!({
            "status": ["open", "in_progress", "resolved"],
            "category": ["IT", "Ops", "Admin", "HR"],
            "priority": ["low", "medium", "high"]
        }),
    )
}

async fn list_assignee_suggestions(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    Query(query): Query<AssigneeSuggestionsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user = require_authenticated_user(&state, &session, &headers).await?;

    let Some(domain) = email_domain(&user.email) else {
        return Ok(response::ok(
            StatusCode::OK,
            Vec::<AssigneeSuggestionRow>::new(),
        ));
    };

    let term = query.q.as_deref().map(str::trim).unwrap_or("");
    let limit = query.limit.unwrap_or(50).clamp(1, 200) as i64;

    let suggestions = sqlx::query_as::<_, AssigneeSuggestionRow>(
        "SELECT
            u.id,
            u.email AS email,
            u.display_name,
            COUNT(req.id)::bigint AS assignment_count
         FROM app.app_users u
         LEFT JOIN app.requests req
           ON req.assignee_user_id = u.id
         WHERE u.email IS NOT NULL
           AND u.id <> $3
           AND lower(split_part(u.email, '@', 2)) = lower($1)
           AND (
             $2::text = ''
             OR NOT EXISTS (
               SELECT 1
               FROM unnest(regexp_split_to_array(lower($2), '[[:space:]]+')) AS t(term)
               WHERE term <> ''
                 AND concat_ws(
                   ' ',
                   lower(u.email),
                   lower(COALESCE(u.display_name, ''))
                 ) NOT LIKE ('%' || term || '%')
             )
           )
         GROUP BY u.id, u.email, u.display_name
         ORDER BY assignment_count DESC, u.email ASC
         LIMIT $4",
    )
    .bind(&domain)
    .bind(term)
    .bind(user.id)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;

    Ok(response::ok(StatusCode::OK, suggestions))
}

async fn list_requests(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    Query(query): Query<ListRequestsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user = require_authenticated_user(&state, &session, &headers).await?;

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = ((page - 1) * limit) as i64;

    if let Some(status) = &query.status {
        validate_status(status)?;
    }
    if let Some(category) = &query.category {
        validate_category(category)?;
    }
    if let Some(priority) = &query.priority {
        validate_priority(priority)?;
    }

    let sort_clause = match query.sort.as_deref() {
        Some("created_at") => "req.created_at ASC",
        Some("updated_at") => "req.updated_at ASC",
        Some("-updated_at") => "req.updated_at DESC",
        _ => "req.created_at DESC",
    };

    let status_filter = query.status.clone();
    let category_filter = query.category.clone();
    let priority_filter = query.priority.clone();
    let search_filter = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);

    let list_query = format!(
        "SELECT {}
         FROM app.request_participants participants
         JOIN app.requests req ON req.id = participants.request_id
         LEFT JOIN app.app_users assignee ON assignee.id = req.assignee_user_id
         WHERE participants.user_id = $1
           AND ($2::text IS NULL OR req.status = $2)
           AND ($3::text IS NULL OR req.category = $3)
           AND ($4::text IS NULL OR req.priority = $4)
           AND (
             $5::text IS NULL
             OR NOT EXISTS (
               SELECT 1
               FROM unnest(regexp_split_to_array(lower($5), '[[:space:]]+')) AS t(term)
               WHERE term <> ''
                 AND concat_ws(
                   ' ',
                   lower(req.title),
                   lower(COALESCE(req.description, '')),
                   lower(req.category),
                   lower(req.status),
                   lower(req.priority),
                   lower(COALESCE(assignee.email, '')),
                   lower(COALESCE(assignee.display_name, ''))
                 ) NOT LIKE ('%' || term || '%')
             )
           )
         ORDER BY {sort_clause}
         LIMIT $6 OFFSET $7",
        request_projection_sql()
    );

    let items = sqlx::query_as::<_, RequestRow>(&list_query)
        .bind(user.id)
        .bind(status_filter.clone())
        .bind(category_filter.clone())
        .bind(priority_filter.clone())
        .bind(search_filter.clone())
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM app.request_participants participants
         JOIN app.requests req ON req.id = participants.request_id
         LEFT JOIN app.app_users assignee ON assignee.id = req.assignee_user_id
         WHERE participants.user_id = $1
           AND ($2::text IS NULL OR req.status = $2)
           AND ($3::text IS NULL OR req.category = $3)
           AND ($4::text IS NULL OR req.priority = $4)
           AND (
             $5::text IS NULL
             OR NOT EXISTS (
               SELECT 1
               FROM unnest(regexp_split_to_array(lower($5), '[[:space:]]+')) AS t(term)
               WHERE term <> ''
                 AND concat_ws(
                   ' ',
                   lower(req.title),
                   lower(COALESCE(req.description, '')),
                   lower(req.category),
                   lower(req.status),
                   lower(req.priority),
                   lower(COALESCE(assignee.email, '')),
                   lower(COALESCE(assignee.display_name, ''))
                 ) NOT LIKE ('%' || term || '%')
             )
           )",
    )
    .bind(user.id)
    .bind(status_filter)
    .bind(category_filter)
    .bind(priority_filter)
    .bind(search_filter)
    .fetch_one(&state.db)
    .await?;

    Ok(response::list(items, page, limit, total as u64))
}

async fn create_request(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    Json(input): Json<CreateRequestInput>,
) -> Result<impl IntoResponse, AppError> {
    validate_create_input(&input)?;

    let user = require_authenticated_user(&state, &session, &headers).await?;
    middleware::require_csrf_token(&state, &session, user.id, &headers).await?;
    let normalized_assignee_email =
        normalize_assignee_email(input.assignee_email.as_deref())?;
    let assignee_user_id = resolve_assignee_user_id(
        &state.db,
        normalized_assignee_email.as_deref(),
    )
    .await?
    .unwrap_or(user.id);

    let request_id: Uuid = sqlx::query_scalar(
        "INSERT INTO app.requests (
            owner_user_id,
            title,
            description,
            category,
            status,
            priority,
            assignee_user_id
         )
         VALUES ($1, $2, $3, $4, 'open', $5, $6)
         RETURNING id",
    )
    .bind(user.id)
    .bind(input.title.trim())
    .bind(input.description.as_deref())
    .bind(input.category)
    .bind(input.priority)
    .bind(assignee_user_id)
    .fetch_one(&state.db)
    .await?;
    let record = fetch_owned_request(&state.db, request_id, user.id).await?;

    let audit_entry = insert_audit_log(
        &state.db,
        record.id,
        user.id,
        "created",
        json!({}),
        json!({
            "title": record.title,
            "status": record.status,
            "category": record.category,
            "priority": record.priority,
            "assignee_email": record.assignee_email,
        }),
    )
    .await?;

    let recipients = fetch_request_recipient_ids(&state.db, record.id).await?;
    publish_event(
        &state,
        &recipients,
        "request.created",
        Some(record.id),
        json!(RequestCreatedEventPayload {
            request: record.clone(),
        }),
    )
    .await;
    publish_event(
        &state,
        &recipients,
        "audit.append",
        Some(record.id),
        json!(AuditAppendEventPayload { audit: audit_entry }),
    )
    .await;

    Ok(response::ok(StatusCode::CREATED, record))
}

async fn get_request(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let user = require_authenticated_user(&state, &session, &headers).await?;

    let item = match fetch_visible_request(&state.db, id, user.id).await {
        Ok(item) => item,
        Err(AppError::NotFound(message)) => {
            warn!(
                request_id = %id,
                user_id = %user.id,
                user_email = %user.email,
                "visible request lookup failed in get_request: {message}"
            );
            return Err(AppError::NotFound(message));
        }
        Err(other) => return Err(other),
    };
    Ok(response::ok(StatusCode::OK, item))
}

async fn update_request(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateRequestInput>,
) -> Result<impl IntoResponse, AppError> {
    let user = require_authenticated_user(&state, &session, &headers).await?;
    middleware::require_csrf_token(&state, &session, user.id, &headers).await?;

    let existing = fetch_editable_request(&state.db, id, user.id).await?;
    let recipients_before = fetch_request_recipient_ids(&state.db, id).await?;

    if let Some(category) = &input.category {
        validate_category(category)?;
    }
    if let Some(priority) = &input.priority {
        validate_priority(priority)?;
    }
    if let Some(status) = &input.status {
        validate_status(status)?;
    }

    let next_title = input
        .title
        .as_deref()
        .map(str::trim)
        .unwrap_or(existing.title.as_str())
        .to_string();

    if next_title.is_empty() {
        return Err(AppError::Validation(vec![ErrorDetail {
            field: "title".to_string(),
            message: "title cannot be empty".to_string(),
        }]));
    }

    let next_description = input.description.or(existing.description.clone());
    let next_category = input.category.unwrap_or(existing.category.clone());
    let next_status = input.status.unwrap_or(existing.status.clone());
    let next_priority = input.priority.unwrap_or(existing.priority.clone());
    let next_assignee_user_id = if let Some(raw_assignee_email) =
        input.assignee_email
    {
        let normalized = normalize_assignee_email(Some(&raw_assignee_email))?;
        resolve_assignee_user_id(&state.db, normalized.as_deref()).await?
    } else {
        existing.assignee_user_id
    };

    let updated_id: Uuid = sqlx::query_scalar(
        "UPDATE app.requests
         SET title = $2,
             description = $3,
             category = $4,
             status = $5,
             priority = $6,
             assignee_user_id = $7,
             updated_at = NOW()
         WHERE id = $1
         RETURNING id",
    )
    .bind(id)
    .bind(&next_title)
    .bind(next_description.as_deref())
    .bind(&next_category)
    .bind(&next_status)
    .bind(&next_priority)
    .bind(next_assignee_user_id)
    .fetch_one(&state.db)
    .await?;
    let updated = fetch_visible_request(&state.db, updated_id, user.id).await?;

    let changed_fields = collect_changed_fields(&existing, &updated);

    let audit_entry = insert_audit_log(
        &state.db,
        updated.id,
        user.id,
        "updated",
        json!({
            "title": existing.title,
            "description": existing.description,
            "category": existing.category,
            "status": existing.status,
            "priority": existing.priority,
            "assignee_email": existing.assignee_email,
        }),
        json!({
            "title": updated.title,
            "description": updated.description,
            "category": updated.category,
            "status": updated.status,
            "priority": updated.priority,
            "assignee_email": updated.assignee_email,
        }),
    )
    .await?;

    let recipients_after =
        fetch_request_recipient_ids(&state.db, updated.id).await?;
    let (existing_recipients, newly_visible_recipients) =
        split_recipients_by_visibility(&recipients_before, &recipients_after);

    publish_event(
        &state,
        &existing_recipients,
        "request.patch",
        Some(updated.id),
        json!(RequestPatchEventPayload {
            request: updated.clone(),
            changed_fields,
            previous_status: existing.status.clone(),
        }),
    )
    .await;
    publish_event(
        &state,
        &newly_visible_recipients,
        "request.created",
        Some(updated.id),
        json!(RequestCreatedEventPayload {
            request: updated.clone(),
        }),
    )
    .await;
    publish_event(
        &state,
        &recipients_after,
        "audit.append",
        Some(updated.id),
        json!(AuditAppendEventPayload { audit: audit_entry }),
    )
    .await;

    Ok(response::ok(StatusCode::OK, updated))
}

async fn delete_request(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let user = require_authenticated_user(&state, &session, &headers).await?;
    middleware::require_csrf_token(&state, &session, user.id, &headers).await?;

    let existing = fetch_owned_request(&state.db, id, user.id).await?;
    let recipients =
        fetch_request_recipient_ids(&state.db, existing.id).await?;

    let audit_entry = insert_audit_log(
        &state.db,
        existing.id,
        user.id,
        "deleted",
        json!({
            "title": existing.title,
            "description": existing.description,
            "category": existing.category,
            "status": existing.status,
            "priority": existing.priority,
            "assignee_email": existing.assignee_email,
        }),
        json!({}),
    )
    .await?;

    sqlx::query("DELETE FROM app.requests WHERE id = $1")
        .bind(existing.id)
        .execute(&state.db)
        .await?;

    publish_event(
        &state,
        &recipients,
        "audit.append",
        Some(existing.id),
        json!(AuditAppendEventPayload { audit: audit_entry }),
    )
    .await;
    publish_event(
        &state,
        &recipients,
        "request.deleted",
        Some(existing.id),
        json!(RequestDeletedEventPayload {
            id: existing.id,
            status: existing.status,
        }),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_request_audit(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let user = require_authenticated_user(&state, &session, &headers).await?;

    match fetch_visible_request(&state.db, id, user.id).await {
        Ok(_) => {}
        Err(AppError::NotFound(message)) => {
            warn!(
                request_id = %id,
                user_id = %user.id,
                user_email = %user.email,
                "visible request lookup failed in get_request_audit: {message}"
            );
            return Err(AppError::NotFound(message));
        }
        Err(other) => return Err(other),
    }

    let items = sqlx::query_as::<_, AuditLogRow>(
        "SELECT
            logs.id,
            logs.request_id,
            logs.actor_user_id,
            COALESCE(actor.email, logs.actor_user_id::text) AS actor_email,
            logs.action,
            logs.old_value,
            logs.new_value,
            logs.created_at
         FROM app.request_audit_logs logs
         LEFT JOIN app.app_users actor ON actor.id = logs.actor_user_id
         WHERE logs.request_id = $1
         ORDER BY logs.created_at DESC",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(response::ok(StatusCode::OK, items))
}

async fn require_authenticated_user(
    state: &AppState,
    session: &Session,
    headers: &HeaderMap,
) -> Result<AuthUserRow, AppError> {
    let context =
        middleware::resolve_request_auth(state, session, headers).await?;
    Ok(AuthUserRow {
        id: context.user.id,
        email: context.user.email,
        display_name: context.user.display_name,
    })
}

async fn fetch_or_create_preferences(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<PreferencesResponse, AppError> {
    sqlx::query_as::<_, PreferencesResponse>(
        "WITH inserted AS (
           INSERT INTO app.user_preferences (user_id)
           VALUES ($1)
           ON CONFLICT (user_id) DO NOTHING
           RETURNING email_digest, browser_alerts, default_page_size
         )
         SELECT email_digest, browser_alerts, default_page_size
         FROM inserted
         UNION ALL
         SELECT email_digest, browser_alerts, default_page_size
         FROM app.user_preferences
         WHERE user_id = $1
         LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        AppError::Internal("failed to load user preferences".to_string())
    })
}

async fn upsert_preferences(
    pool: &PgPool,
    user_id: Uuid,
    update: &PreferencesUpdate,
) -> Result<PreferencesResponse, AppError> {
    sqlx::query_as::<_, PreferencesResponse>(
        "INSERT INTO app.user_preferences (
           user_id,
           email_digest,
           browser_alerts,
           default_page_size
         )
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id) DO UPDATE SET
           email_digest = EXCLUDED.email_digest,
           browser_alerts = EXCLUDED.browser_alerts,
           default_page_size = EXCLUDED.default_page_size
         RETURNING email_digest, browser_alerts, default_page_size",
    )
    .bind(user_id)
    .bind(update.email_digest)
    .bind(update.browser_alerts)
    .bind(update.default_page_size)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

async fn fetch_owned_request(
    pool: &PgPool,
    request_id: Uuid,
    owner_id: Uuid,
) -> Result<RequestRow, AppError> {
    let query = format!(
        "SELECT {}
         FROM app.requests req
         LEFT JOIN app.app_users assignee ON assignee.id = req.assignee_user_id
         WHERE req.id = $1 AND req.owner_user_id = $2",
        request_projection_sql()
    );

    sqlx::query_as::<_, RequestRow>(&query)
        .bind(request_id)
        .bind(owner_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("request not found".to_string()))
}

async fn fetch_editable_request(
    pool: &PgPool,
    request_id: Uuid,
    user_id: Uuid,
) -> Result<RequestRow, AppError> {
    let query = format!(
        "SELECT {}
         FROM app.requests req
         LEFT JOIN app.app_users assignee ON assignee.id = req.assignee_user_id
         WHERE req.id = $1
           AND (req.owner_user_id = $2 OR req.assignee_user_id = $2)",
        request_projection_sql()
    );

    sqlx::query_as::<_, RequestRow>(&query)
        .bind(request_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("request not found".to_string()))
}

async fn fetch_visible_request(
    pool: &PgPool,
    request_id: Uuid,
    user_id: Uuid,
) -> Result<RequestRow, AppError> {
    let query = format!(
        "SELECT {}
         FROM app.requests req
         JOIN app.request_participants participants
           ON participants.request_id = req.id
          AND participants.user_id = $2
         LEFT JOIN app.app_users assignee ON assignee.id = req.assignee_user_id
         WHERE req.id = $1",
        request_projection_sql()
    );

    sqlx::query_as::<_, RequestRow>(&query)
        .bind(request_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("request not found".to_string()))
}

fn collect_changed_fields(
    existing: &RequestRow,
    updated: &RequestRow,
) -> Vec<String> {
    let mut fields = Vec::new();

    if existing.title != updated.title {
        fields.push("title".to_string());
    }
    if existing.description != updated.description {
        fields.push("description".to_string());
    }
    if existing.category != updated.category {
        fields.push("category".to_string());
    }
    if existing.status != updated.status {
        fields.push("status".to_string());
    }
    if existing.priority != updated.priority {
        fields.push("priority".to_string());
    }
    if existing.assignee_user_id != updated.assignee_user_id {
        fields.push("assignee_user_id".to_string());
        fields.push("assignee_email".to_string());
        fields.push("assignee_display_name".to_string());
    }
    if existing.updated_at != updated.updated_at {
        fields.push("updated_at".to_string());
    }

    if fields.is_empty() {
        fields.push("updated_at".to_string());
    }

    fields
}

async fn insert_audit_log(
    pool: &PgPool,
    request_id: Uuid,
    actor_user_id: Uuid,
    action: &str,
    old_value: serde_json::Value,
    new_value: serde_json::Value,
) -> Result<AuditLogRow, AppError> {
    sqlx::query_as::<_, AuditLogRow>(
        "WITH inserted AS (
           INSERT INTO app.request_audit_logs (
             request_id,
             actor_user_id,
             action,
             old_value,
             new_value
           )
           VALUES ($1, $2, $3, $4, $5)
           RETURNING
             id,
             request_id,
             actor_user_id,
             action,
             old_value,
             new_value,
             created_at
         )
         SELECT
           inserted.id,
           inserted.request_id,
           inserted.actor_user_id,
           COALESCE(actor.email, inserted.actor_user_id::text) AS actor_email,
           inserted.action,
           inserted.old_value,
           inserted.new_value,
           inserted.created_at
         FROM inserted
         LEFT JOIN app.app_users actor ON actor.id = inserted.actor_user_id",
    )
    .bind(request_id)
    .bind(actor_user_id)
    .bind(action)
    .bind(old_value)
    .bind(new_value)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

async fn fetch_request_recipient_ids(
    pool: &PgPool,
    request_id: Uuid,
) -> Result<Vec<Uuid>, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id
         FROM app.request_participants
         WHERE request_id = $1",
    )
    .bind(request_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

fn split_recipients_by_visibility(
    recipients_before: &[Uuid],
    recipients_after: &[Uuid],
) -> (Vec<Uuid>, Vec<Uuid>) {
    let before_set: HashSet<Uuid> = recipients_before.iter().copied().collect();
    let mut existing = Vec::with_capacity(recipients_after.len());
    let mut newly_visible = Vec::new();

    for recipient_id in recipients_after {
        if before_set.contains(recipient_id) {
            existing.push(*recipient_id);
        } else {
            newly_visible.push(*recipient_id);
        }
    }

    (existing, newly_visible)
}

async fn publish_event(
    state: &AppState,
    recipients: &[Uuid],
    event_type: &str,
    request_id: Option<Uuid>,
    payload: serde_json::Value,
) {
    if recipients.is_empty() {
        return;
    }

    let message = match EventEnvelope::new(
        event_type.to_string(),
        request_id,
        None,
        payload,
    )
    .encode()
    {
        Ok(encoded) => encoded,
        Err(error) => {
            warn!(
                event_type,
                request_id = ?request_id,
                error = %error,
                "failed to encode websocket event",
            );
            return;
        }
    };

    let delivered = state
        .realtime_hub
        .broadcast_to_users(recipients.iter().copied(), message)
        .await;

    if delivered == 0 {
        warn!(
            event_type,
            request_id = ?request_id,
            recipients = recipients.len(),
            "websocket event had no connected recipients",
        );
    }
}

fn request_projection_sql() -> &'static str {
    "req.id,
     req.owner_user_id,
     req.title,
     req.description,
     req.category,
     req.status,
     req.priority,
     req.assignee_user_id,
     assignee.email AS assignee_email,
     CASE
       WHEN assignee.id IS NULL THEN NULL
       ELSE COALESCE(
         NULLIF(assignee.display_name, ''),
         split_part(assignee.email, '@', 1),
         'user'
       )
     END AS assignee_display_name,
     req.created_at,
     req.updated_at"
}

async fn resolve_assignee_user_id(
    pool: &PgPool,
    assignee_email: Option<&str>,
) -> Result<Option<Uuid>, AppError> {
    let Some(email) = assignee_email else {
        return Ok(None);
    };

    let user_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id
         FROM app.app_users
         WHERE lower(email) = lower($1)
         LIMIT 1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    user_id
        .ok_or_else(|| {
            AppError::Validation(vec![ErrorDetail {
                field: "assignee_email".to_string(),
                message: "No user exists with this email address".to_string(),
            }])
        })
        .map(Some)
}

fn normalize_assignee_email(
    raw: Option<&str>,
) -> Result<Option<String>, AppError> {
    let Some(value) = raw else {
        return Ok(None);
    };

    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    if !is_valid_email(trimmed) {
        return Err(AppError::Validation(vec![ErrorDetail {
            field: "assignee_email".to_string(),
            message: "assignee_email must be a valid email address".to_string(),
        }]));
    }

    Ok(Some(trimmed.to_lowercase()))
}

fn is_valid_email(value: &str) -> bool {
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return false;
    }

    if value.matches('@').count() != 1 {
        return false;
    }

    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };

    !local.is_empty()
        && !domain.is_empty()
        && !domain.starts_with('.')
        && !domain.ends_with('.')
}

fn email_domain(email: &str) -> Option<String> {
    let (_, domain) = email.split_once('@')?;
    let normalized = domain.trim().to_lowercase();
    if normalized.is_empty() {
        return None;
    }
    Some(normalized)
}

fn normalize_display_name(raw: Option<&str>) -> Result<String, AppError> {
    let Some(value) = raw else {
        return Err(AppError::Validation(vec![ErrorDetail {
            field: "display_name".to_string(),
            message: "display_name is required".to_string(),
        }]));
    };

    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(vec![ErrorDetail {
            field: "display_name".to_string(),
            message: "display_name is required".to_string(),
        }]));
    }

    if trimmed.chars().count() > 120 {
        return Err(AppError::Validation(vec![ErrorDetail {
            field: "display_name".to_string(),
            message: "display_name must be <= 120 characters".to_string(),
        }]));
    }

    Ok(trimmed.to_string())
}

fn normalize_preferences_update(
    input: UpdatePreferencesInput,
    current: &PreferencesResponse,
) -> Result<PreferencesUpdate, AppError> {
    let default_page_size =
        input.default_page_size.unwrap_or(current.default_page_size);
    if !matches!(default_page_size, 10 | 20 | 50 | 100) {
        return Err(AppError::Validation(vec![ErrorDetail {
            field: "default_page_size".to_string(),
            message: "default_page_size must be one of 10, 20, 50, 100"
                .to_string(),
        }]));
    }

    Ok(PreferencesUpdate {
        email_digest: input.email_digest.unwrap_or(current.email_digest),
        browser_alerts: input.browser_alerts.unwrap_or(current.browser_alerts),
        default_page_size,
    })
}

fn validate_create_input(input: &CreateRequestInput) -> Result<(), AppError> {
    let mut details = Vec::new();

    if input.title.trim().is_empty() {
        details.push(ErrorDetail {
            field: "title".to_string(),
            message: "title is required".to_string(),
        });
    }

    if input.title.len() > 255 {
        details.push(ErrorDetail {
            field: "title".to_string(),
            message: "title must be <= 255 characters".to_string(),
        });
    }

    if let Some(description) = &input.description
        && description.len() > 5000
    {
        details.push(ErrorDetail {
            field: "description".to_string(),
            message: "description must be <= 5000 characters".to_string(),
        });
    }

    if validate_category(&input.category).is_err() {
        details.push(ErrorDetail {
            field: "category".to_string(),
            message: "category must be one of IT, Ops, Admin, HR".to_string(),
        });
    }

    if validate_priority(&input.priority).is_err() {
        details.push(ErrorDetail {
            field: "priority".to_string(),
            message: "priority must be one of low, medium, high".to_string(),
        });
    }

    if let Some(assignee_email) = input.assignee_email.as_deref()
        && !assignee_email.trim().is_empty()
        && !is_valid_email(assignee_email.trim())
    {
        details.push(ErrorDetail {
            field: "assignee_email".to_string(),
            message: "assignee_email must be a valid email address".to_string(),
        });
    }

    if details.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(details))
    }
}

fn validate_status(value: &str) -> Result<(), AppError> {
    match value {
        "open" | "in_progress" | "resolved" => Ok(()),
        _ => Err(AppError::Validation(vec![ErrorDetail {
            field: "status".to_string(),
            message: "status must be one of open, in_progress, resolved"
                .to_string(),
        }])),
    }
}

fn validate_category(value: &str) -> Result<(), AppError> {
    match value {
        "IT" | "Ops" | "Admin" | "HR" => Ok(()),
        _ => Err(AppError::Validation(vec![ErrorDetail {
            field: "category".to_string(),
            message: "category must be one of IT, Ops, Admin, HR".to_string(),
        }])),
    }
}

fn validate_priority(value: &str) -> Result<(), AppError> {
    match value {
        "low" | "medium" | "high" => Ok(()),
        _ => Err(AppError::Validation(vec![ErrorDetail {
            field: "priority".to_string(),
            message: "priority must be one of low, medium, high".to_string(),
        }])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_status_rejects_invalid() {
        let err = validate_status("bad").expect_err("status should fail");
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_priority_accepts_valid() {
        validate_priority("low").expect("low should pass");
        validate_priority("medium").expect("medium should pass");
        validate_priority("high").expect("high should pass");
    }

    #[test]
    fn validate_create_input_rejects_empty_title() {
        let input = CreateRequestInput {
            title: " ".to_string(),
            description: None,
            category: "IT".to_string(),
            priority: "low".to_string(),
            assignee_email: None,
        };

        let err =
            validate_create_input(&input).expect_err("empty title should fail");
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_category_rejects_invalid() {
        let err = validate_category("Finance").expect_err("invalid category");
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_create_input_rejects_long_description() {
        let input = CreateRequestInput {
            title: "Need access".to_string(),
            description: Some("x".repeat(5001)),
            category: "IT".to_string(),
            priority: "medium".to_string(),
            assignee_email: None,
        };

        let err = validate_create_input(&input)
            .expect_err("long description should fail");
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_create_input_rejects_invalid_assignee_email() {
        let input = CreateRequestInput {
            title: "Need access".to_string(),
            description: None,
            category: "IT".to_string(),
            priority: "medium".to_string(),
            assignee_email: Some("invalid-email".to_string()),
        };

        let err = validate_create_input(&input)
            .expect_err("invalid assignee email should fail");
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn normalize_display_name_rejects_missing_value() {
        let err = normalize_display_name(None)
            .expect_err("missing display name should fail");
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn normalize_display_name_rejects_whitespace_value() {
        let err = normalize_display_name(Some("   "))
            .expect_err("blank display name should fail");
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn normalize_display_name_trims_valid_value() {
        let value = normalize_display_name(Some("  Team Ops  "))
            .expect("display name should normalize");
        assert_eq!(value, "Team Ops");
    }

    #[test]
    fn normalize_preferences_update_rejects_invalid_page_size() {
        let current = PreferencesResponse {
            email_digest: true,
            browser_alerts: true,
            default_page_size: 20,
        };

        let err = normalize_preferences_update(
            UpdatePreferencesInput {
                email_digest: None,
                browser_alerts: None,
                default_page_size: Some(33),
            },
            &current,
        )
        .expect_err("invalid page size should fail");

        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn normalize_preferences_update_uses_current_for_missing_fields() {
        let current = PreferencesResponse {
            email_digest: true,
            browser_alerts: false,
            default_page_size: 20,
        };

        let update = normalize_preferences_update(
            UpdatePreferencesInput {
                email_digest: Some(false),
                browser_alerts: None,
                default_page_size: Some(50),
            },
            &current,
        )
        .expect("preferences should normalize");

        assert!(!update.email_digest);
        assert!(!update.browser_alerts);
        assert_eq!(update.default_page_size, 50);
    }

    #[test]
    fn split_recipients_by_visibility_partitions_newly_visible_users() {
        let user_a = Uuid::new_v4();
        let user_b = Uuid::new_v4();
        let user_c = Uuid::new_v4();

        let before = vec![user_a, user_b];
        let after = vec![user_a, user_b, user_c];

        let (existing, newly_visible) =
            split_recipients_by_visibility(&before, &after);

        assert_eq!(existing, vec![user_a, user_b]);
        assert_eq!(newly_visible, vec![user_c]);
    }
}
