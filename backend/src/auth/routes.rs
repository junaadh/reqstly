use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use tower_sessions::Session;

use crate::{
    AppState,
    auth::{
        middleware, service,
        types::{
            PasskeyLoginFinishRequest, PasskeyLoginStartRequest,
            PasskeyRegisterFinishRequest, PasskeyRegisterStartRequest,
            PasskeySignupFinishRequest, PasskeySignupStartRequest,
            PasswordLoginRequest, SignupRequest,
        },
    },
    error::AppError,
    response,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/signup", post(signup))
        .route("/auth/login/password", post(login_password))
        .route("/auth/csrf", get(issue_csrf_token))
        .route("/auth/logout", post(logout))
        .route("/auth/sessions/revoke", post(revoke_all_sessions))
        .route("/auth/ws-token", post(issue_ws_token))
        .route("/auth/passkeys", get(passkeys_list))
        .route(
            "/auth/passkeys/register/start",
            post(passkey_register_start),
        )
        .route(
            "/auth/passkeys/register/finish",
            post(passkey_register_finish),
        )
        .route("/auth/passkeys/signup/start", post(passkey_signup_start))
        .route("/auth/passkeys/signup/finish", post(passkey_signup_finish))
        .route("/auth/passkeys/login/start", post(passkey_login_start))
        .route("/auth/passkeys/login/finish", post(passkey_login_finish))
}

async fn signup(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    Json(payload): Json<SignupRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let user = service::signup(&state, &session, &headers, payload).await?;
    Ok(response::ok(StatusCode::CREATED, user))
}

async fn login_password(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    Json(payload): Json<PasswordLoginRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let user =
        service::login_password(&state, &session, &headers, payload).await?;
    Ok(response::ok(StatusCode::OK, user))
}

async fn logout(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let auth =
        middleware::resolve_request_auth(&state, &session, &headers).await?;
    middleware::require_csrf_token(&state, &session, auth.user.id, &headers)
        .await?;
    service::logout(&state, &session, &headers, Some(&auth)).await?;
    Ok(response::ok(
        StatusCode::OK,
        serde_json::json!({ "ok": true }),
    ))
}

async fn issue_csrf_token(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let auth =
        middleware::resolve_request_auth(&state, &session, &headers).await?;
    let token =
        service::issue_csrf_token(&state, &session, &headers, &auth).await?;
    Ok(response::ok(StatusCode::OK, token))
}

async fn issue_ws_token(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let auth =
        middleware::resolve_request_auth(&state, &session, &headers).await?;
    middleware::require_csrf_token(&state, &session, auth.user.id, &headers)
        .await?;
    let token = service::issue_ws_token(&state, &headers, &auth).await?;
    Ok(response::ok(StatusCode::OK, token))
}

async fn revoke_all_sessions(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let auth =
        middleware::resolve_request_auth(&state, &session, &headers).await?;
    middleware::require_csrf_token(&state, &session, auth.user.id, &headers)
        .await?;
    service::revoke_all_sessions(&state, &session, &headers, &auth).await?;
    Ok(response::ok(
        StatusCode::OK,
        serde_json::json!({ "ok": true }),
    ))
}

async fn passkeys_list(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let auth =
        middleware::resolve_request_auth(&state, &session, &headers).await?;
    let passkeys = service::list_passkeys(&state, &auth).await?;
    Ok(response::ok(StatusCode::OK, passkeys))
}

async fn passkey_register_start(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    Json(payload): Json<PasskeyRegisterStartRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let auth =
        middleware::resolve_request_auth(&state, &session, &headers).await?;
    middleware::require_csrf_token(&state, &session, auth.user.id, &headers)
        .await?;
    let challenge =
        service::start_passkey_registration(&state, &auth, &headers, payload)
            .await?;
    Ok(response::ok(StatusCode::OK, challenge))
}

async fn passkey_register_finish(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    Json(payload): Json<PasskeyRegisterFinishRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let auth =
        middleware::resolve_request_auth(&state, &session, &headers).await?;
    middleware::require_csrf_token(&state, &session, auth.user.id, &headers)
        .await?;
    let user =
        service::finish_passkey_registration(&state, &auth, &headers, payload)
            .await?;
    Ok(response::ok(StatusCode::OK, user))
}

async fn passkey_login_start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PasskeyLoginStartRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let challenge =
        service::start_passkey_login(&state, &headers, payload).await?;
    Ok(response::ok(StatusCode::OK, challenge))
}

async fn passkey_signup_start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PasskeySignupStartRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let challenge =
        service::start_passkey_signup(&state, &headers, payload).await?;
    Ok(response::ok(StatusCode::OK, challenge))
}

async fn passkey_signup_finish(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    Json(payload): Json<PasskeySignupFinishRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let user =
        service::finish_passkey_signup(&state, &session, &headers, payload)
            .await?;
    Ok(response::ok(StatusCode::CREATED, user))
}

async fn passkey_login_finish(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    Json(payload): Json<PasskeyLoginFinishRequest>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let user =
        service::finish_passkey_login(&state, &session, &headers, payload)
            .await?;
    Ok(response::ok(StatusCode::OK, user))
}
