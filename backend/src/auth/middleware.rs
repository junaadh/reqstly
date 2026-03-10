use axum::http::{HeaderMap, header};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tower_sessions::Session;
use uuid::Uuid;

use crate::{
    AppState,
    auth::{repo, session},
    error::AppError,
};

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user: crate::auth::types::AuthUserProfile,
}

#[derive(Debug, Deserialize)]
struct WsClaims {
    sub: String,
    aud: String,
    iss: String,
    #[serde(rename = "exp")]
    _exp: usize,
}

pub async fn resolve_request_auth(
    state: &AppState,
    session_handle: &Session,
    headers: &HeaderMap,
) -> Result<AuthContext, AppError> {
    if let Some(session_user) =
        session::load_session_user(session_handle).await?
    {
        let security =
            repo::ensure_user_auth_security(&state.db, session_user.user_id)
                .await?;

        if security.compromised_at.is_some()
            || security.require_reauth
            || security.locked_until.is_some_and(|locked_until| {
                locked_until > time::OffsetDateTime::now_utc()
            })
            || security.session_version != session_user.session_version
        {
            let _ = session::clear_session(session_handle).await;
            return Err(AppError::Unauthorized(
                "session is no longer valid".to_string(),
            ));
        }

        let user = repo::get_user_by_id(&state.db, session_user.user_id)
            .await?
            .ok_or_else(|| {
                AppError::Unauthorized("session user not found".to_string())
            })?;

        if !user.is_active {
            return Err(AppError::Unauthorized(
                "account is disabled".to_string(),
            ));
        }

        return Ok(AuthContext { user });
    }

    let token = extract_bearer_token(headers)?;
    let user_id = verify_ws_token_with_state(state, token).await?;

    let user =
        repo::get_user_by_id(&state.db, user_id)
            .await?
            .ok_or_else(|| {
                AppError::Unauthorized("token user not found".to_string())
            })?;

    if !user.is_active {
        return Err(AppError::Unauthorized("account is disabled".to_string()));
    }

    Ok(AuthContext { user })
}

pub async fn verify_ws_token_with_state(
    state: &AppState,
    token: &str,
) -> Result<Uuid, AppError> {
    let user_id =
        verify_ws_token(token, &state.ws_token_secret, &state.ws_token_issuer)?;

    let token_fingerprint = Sha256::digest(token.as_bytes()).to_vec();
    let is_active = repo::is_ws_token_issuance_active(
        &state.db,
        user_id,
        token_fingerprint.as_slice(),
    )
    .await?;

    if !is_active {
        return Err(AppError::Unauthorized(
            "invalid or expired token".to_string(),
        ));
    }

    repo::mark_ws_token_used(&state.db, user_id, token_fingerprint.as_slice())
        .await?;

    let security = repo::ensure_user_auth_security(&state.db, user_id).await?;
    if security.compromised_at.is_some()
        || security.require_reauth
        || security.locked_until.is_some_and(|locked_until| {
            locked_until > time::OffsetDateTime::now_utc()
        })
    {
        return Err(AppError::Unauthorized(
            "token user is temporarily restricted".to_string(),
        ));
    }

    Ok(user_id)
}

pub async fn require_csrf_token(
    state: &AppState,
    session_handle: &Session,
    user_id: Uuid,
    headers: &HeaderMap,
) -> Result<(), AppError> {
    let uses_session_cookie =
        session::load_session_user(session_handle).await?.is_some();
    if !uses_session_cookie {
        return Ok(());
    }

    let token = headers
        .get("x-csrf-token")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::Unauthorized("missing csrf token".to_string())
        })?;

    let origin = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok());
    if origin.is_some()
        && !crate::realtime::is_origin_allowed(
            origin,
            &state.ws_allowed_origins,
        )
    {
        return Err(AppError::Unauthorized(
            "invalid csrf request origin".to_string(),
        ));
    }

    let session_id = session::session_id(session_handle).ok_or_else(|| {
        AppError::Unauthorized("missing active session".to_string())
    })?;
    let token_hash = Sha256::digest(token.as_bytes()).to_vec();

    let valid = repo::is_csrf_token_valid(
        &state.db,
        &session_id,
        user_id,
        "session",
        token_hash.as_slice(),
    )
    .await?;

    if !valid {
        return Err(AppError::Unauthorized("invalid csrf token".to_string()));
    }

    Ok(())
}

pub fn extract_bearer_token(headers: &HeaderMap) -> Result<&str, AppError> {
    let authorization = headers
        .get(header::AUTHORIZATION)
        .ok_or_else(|| {
            AppError::Unauthorized("missing Authorization header".to_string())
        })?
        .to_str()
        .map_err(|_| {
            AppError::Unauthorized("invalid Authorization header".to_string())
        })?;

    authorization.strip_prefix("Bearer ").ok_or_else(|| {
        AppError::Unauthorized("expected Bearer token".to_string())
    })
}

pub fn verify_ws_token(
    token: &str,
    secret: &str,
    issuer: &str,
) -> Result<Uuid, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[issuer]);
    validation.set_audience(&["ws"]);

    let decoded = decode::<WsClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|_| {
        AppError::Unauthorized("invalid or expired token".to_string())
    })?;

    if decoded.claims.aud != "ws" || decoded.claims.iss != issuer {
        return Err(AppError::Unauthorized(
            "invalid token audience or issuer".to_string(),
        ));
    }

    Uuid::parse_str(&decoded.claims.sub).map_err(|_| {
        AppError::Unauthorized("token subject is not a UUID".to_string())
    })
}
