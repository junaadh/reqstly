use axum::http::HeaderMap;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime};
use tower_sessions::Session;
use uuid::Uuid;
use webauthn_rs::prelude::{
    Passkey, PasskeyAuthentication, PasskeyRegistration, PublicKeyCredential,
    RegisterPublicKeyCredential,
};

use crate::{
    AppState,
    auth::{
        errors::validation_error,
        middleware::AuthContext,
        passkey, repo, session,
        types::{
            AuthMethod, AuthUserProfile, CsrfTokenResponse,
            PasskeyChallengeResponse, PasskeyCredentialSummary,
            PasskeyListResponse, PasskeyLoginFinishRequest,
            PasskeyLoginStartRequest, PasskeyRegisterFinishRequest,
            PasskeyRegisterStartRequest, PasskeySignupFinishRequest,
            PasskeySignupStartRequest, PasskeyStats, PasswordLoginRequest,
            SignupRequest, WsTokenResponse,
        },
    },
    error::{AppError, ErrorDetail},
};

#[derive(Debug, Serialize)]
struct WsTokenClaims {
    sub: String,
    aud: String,
    iss: String,
    jti: String,
    exp: usize,
    iat: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingPasskeySignup {
    user_id: Uuid,
    email: String,
    display_name: String,
}

pub async fn signup(
    state: &AppState,
    session_handle: &Session,
    headers: &HeaderMap,
    input: SignupRequest,
) -> Result<AuthUserProfile, AppError> {
    crate::auth::rate_limit::check_auth_rate_limit(state, "signup", headers)
        .await?;

    let email = normalize_email(&input.email)?;
    validate_password(&input.password)?;
    let display_name =
        normalize_display_name(input.display_name.as_deref(), &email)?;

    let password_hash = crate::auth::password::hash_password(&input.password)?;

    let user = repo::create_user_with_password(
        &state.db,
        &email,
        &display_name,
        &password_hash,
    )
    .await?;

    let security = repo::ensure_user_auth_security(&state.db, user.id).await?;

    let _ = session::establish_session(
        session_handle,
        user.id,
        AuthMethod::Password,
        security.session_version,
    )
    .await?;

    repo::update_last_login(&state.db, user.id).await?;
    repo::mark_authentication_success(&state.db, user.id, "password").await?;
    repo::insert_auth_event(
        &state.db,
        Some(user.id),
        "signup.password",
        true,
        read_ip(headers),
        read_user_agent(headers),
        json!({}),
    )
    .await?;

    Ok(user)
}

pub async fn login_password(
    state: &AppState,
    session_handle: &Session,
    headers: &HeaderMap,
    input: PasswordLoginRequest,
) -> Result<AuthUserProfile, AppError> {
    crate::auth::rate_limit::check_auth_rate_limit(
        state,
        "login_password",
        headers,
    )
    .await?;

    let email = normalize_email(&input.email)?;

    let identity =
        repo::find_password_identity_by_email(&state.db, &email).await?;

    let Some(identity) = identity else {
        record_login_failure(state, headers, &email, "identity-not-found")
            .await;
        return Err(AppError::Unauthorized(
            "invalid email or password".to_string(),
        ));
    };

    if identity.deleted_at.is_some() {
        record_login_failure(state, headers, &email, "deleted-account").await;
        return Err(AppError::Unauthorized(
            "invalid email or password".to_string(),
        ));
    }

    if identity
        .locked_until
        .is_some_and(|locked_until| locked_until > OffsetDateTime::now_utc())
    {
        record_login_failure(state, headers, &email, "locked-out").await;
        return Err(AppError::Unauthorized(
            "invalid email or password".to_string(),
        ));
    }

    let security =
        repo::ensure_user_auth_security(&state.db, identity.user_id).await?;

    if security.require_reauth {
        record_login_failure(state, headers, &email, "reauth-required").await;
        return Err(AppError::Unauthorized(
            "invalid email or password".to_string(),
        ));
    }

    if security
        .locked_until
        .is_some_and(|locked_until| locked_until > OffsetDateTime::now_utc())
    {
        record_login_failure(state, headers, &email, "security-locked").await;
        return Err(AppError::Unauthorized(
            "invalid email or password".to_string(),
        ));
    }

    if security.password_login_disabled {
        record_login_failure(state, headers, &email, "password-login-disabled")
            .await;
        return Err(AppError::Unauthorized(
            "invalid email or password".to_string(),
        ));
    }

    let password_valid = crate::auth::password::verify_password(
        &input.password,
        &identity.password_hash,
    )?;

    if !password_valid {
        let _ =
            repo::record_password_login_failure(&state.db, identity.user_id)
                .await;
        record_login_failure(state, headers, &email, "password-mismatch").await;
        return Err(AppError::Unauthorized(
            "invalid email or password".to_string(),
        ));
    }

    if !identity.is_active {
        record_login_failure(state, headers, &email, "inactive-account").await;
        return Err(AppError::Unauthorized("account is disabled".to_string()));
    }

    let _ = session::establish_session(
        session_handle,
        identity.user_id,
        AuthMethod::Password,
        security.session_version,
    )
    .await?;

    repo::clear_password_login_failures(&state.db, identity.user_id).await?;
    repo::update_last_login(&state.db, identity.user_id).await?;
    repo::mark_authentication_success(&state.db, identity.user_id, "password")
        .await?;

    repo::insert_auth_event(
        &state.db,
        Some(identity.user_id),
        "login.password",
        true,
        read_ip(headers),
        read_user_agent(headers),
        json!({}),
    )
    .await?;

    Ok(AuthUserProfile {
        id: identity.user_id,
        email: identity.email,
        display_name: identity.display_name,
        is_active: identity.is_active,
        email_verified: identity.email_verified,
    })
}

pub async fn logout(
    state: &AppState,
    session_handle: &Session,
    headers: &HeaderMap,
    auth: Option<&AuthContext>,
) -> Result<(), AppError> {
    if let Some(context) = auth {
        repo::insert_auth_event(
            &state.db,
            Some(context.user.id),
            "logout",
            true,
            read_ip(headers),
            read_user_agent(headers),
            json!({}),
        )
        .await?;
    }

    session::clear_session(session_handle).await
}

pub async fn revoke_all_sessions(
    state: &AppState,
    session_handle: &Session,
    headers: &HeaderMap,
    auth: &AuthContext,
) -> Result<(), AppError> {
    let revoked_ws_tokens = repo::revoke_ws_tokens_for_user(
        &state.db,
        auth.user.id,
        "session_revoke_all",
    )
    .await?;
    let next_session_version =
        repo::bump_user_session_version(&state.db, auth.user.id).await?;

    repo::insert_auth_event(
        &state.db,
        Some(auth.user.id),
        "session.revoke_all",
        true,
        read_ip(headers),
        read_user_agent(headers),
        json!({
            "session_version": next_session_version,
            "revoked_ws_tokens": revoked_ws_tokens
        }),
    )
    .await?;

    session::clear_session(session_handle).await
}

pub async fn issue_ws_token(
    state: &AppState,
    headers: &HeaderMap,
    auth: &AuthContext,
) -> Result<WsTokenResponse, AppError> {
    let security =
        repo::ensure_user_auth_security(&state.db, auth.user.id).await?;
    let now = OffsetDateTime::now_utc();
    let expires_at = now + Duration::minutes(15);

    let claims = WsTokenClaims {
        sub: auth.user.id.to_string(),
        aud: "ws".to_string(),
        iss: state.ws_token_issuer.clone(),
        jti: Uuid::new_v4().to_string(),
        iat: now.unix_timestamp() as usize,
        exp: expires_at.unix_timestamp() as usize,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(state.ws_token_secret.as_bytes()),
    )
    .map_err(|err| {
        AppError::Internal(format!("failed to issue ws token: {err}"))
    })?;

    let token_fingerprint = Sha256::digest(token.as_bytes()).to_vec();
    repo::create_ws_token_issuance(
        &state.db,
        auth.user.id,
        token_fingerprint.as_slice(),
        expires_at,
        read_ip(headers),
        read_user_agent(headers),
        json!({ "aud": "ws", "session_version": security.session_version }),
    )
    .await?;

    Ok(WsTokenResponse { token, expires_at })
}

pub async fn issue_csrf_token(
    state: &AppState,
    session_handle: &Session,
    headers: &HeaderMap,
    auth: &AuthContext,
) -> Result<CsrfTokenResponse, AppError> {
    let session_id = session::session_id(session_handle).ok_or_else(|| {
        AppError::Unauthorized("missing active session".to_string())
    })?;

    let now = OffsetDateTime::now_utc();
    let expires_at = now + Duration::hours(12);
    let token =
        format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let token_hash = Sha256::digest(token.as_bytes()).to_vec();

    repo::create_csrf_token(
        &state.db,
        &session_id,
        auth.user.id,
        "session",
        token_hash.as_slice(),
        expires_at,
        json!({}),
    )
    .await?;

    repo::insert_auth_event(
        &state.db,
        Some(auth.user.id),
        "csrf.issue",
        true,
        read_ip(headers),
        read_user_agent(headers),
        json!({}),
    )
    .await?;

    Ok(CsrfTokenResponse { token, expires_at })
}

pub async fn list_passkeys(
    state: &AppState,
    auth: &AuthContext,
) -> Result<PasskeyListResponse, AppError> {
    let rows =
        repo::list_user_passkey_summaries(&state.db, auth.user.id).await?;
    let stats = repo::get_user_passkey_stats(&state.db, auth.user.id).await?;

    let credentials = rows
        .into_iter()
        .map(|row| PasskeyCredentialSummary {
            id: row.id,
            nickname: row.nickname,
            created_at: row.created_at,
            first_used_at: row.first_used_at,
            last_used_at: row.last_used_at,
        })
        .collect();

    Ok(PasskeyListResponse {
        credentials,
        stats: PasskeyStats {
            passkey_count: stats.passkey_count,
            first_registered_at: stats.first_registered_at,
            first_used_at: stats.first_used_at,
            last_used_at: stats.last_used_at,
        },
    })
}

pub async fn start_passkey_registration(
    state: &AppState,
    auth: &AuthContext,
    headers: &HeaderMap,
    input: PasskeyRegisterStartRequest,
) -> Result<PasskeyChallengeResponse, AppError> {
    crate::auth::rate_limit::check_auth_rate_limit(
        state,
        "passkey_register_start",
        headers,
    )
    .await?;

    let security =
        repo::ensure_user_auth_security(&state.db, auth.user.id).await?;
    if security.passkey_login_disabled {
        return Err(AppError::Unauthorized(
            "passkey enrollment is disabled for this account".to_string(),
        ));
    }

    let existing = load_user_passkeys(state, auth.user.id).await?;
    let (options, state_blob) = state.passkey.start_registration(
        auth.user.id,
        &auth.user.email,
        &auth.user.display_name,
        &existing,
    )?;

    let expires_at = OffsetDateTime::now_utc() + Duration::minutes(5);
    let challenge_id = repo::create_webauthn_challenge(
        &state.db,
        Some(auth.user.id),
        "register",
        json!({
            "state": passkey::to_json_value(&state_blob)?,
            "nickname": input.nickname
        }),
        expires_at,
    )
    .await?;

    Ok(PasskeyChallengeResponse {
        challenge_id,
        options: passkey::to_json_value(&options)?,
    })
}

pub async fn finish_passkey_registration(
    state: &AppState,
    auth: &AuthContext,
    headers: &HeaderMap,
    input: PasskeyRegisterFinishRequest,
) -> Result<AuthUserProfile, AppError> {
    crate::auth::rate_limit::check_auth_rate_limit(
        state,
        "passkey_register_finish",
        headers,
    )
    .await?;

    let challenge = repo::consume_webauthn_challenge(
        &state.db,
        input.challenge_id,
        "register",
    )
    .await?
    .ok_or_else(|| {
        AppError::Unauthorized(
            "invalid or expired passkey challenge".to_string(),
        )
    })?;

    if challenge.user_id != Some(auth.user.id) {
        return Err(AppError::Unauthorized(
            "invalid passkey challenge owner".to_string(),
        ));
    }

    let state_value = challenge
        .challenge_blob
        .get("state")
        .cloned()
        .ok_or_else(|| {
            AppError::Internal("passkey challenge state missing".to_string())
        })?;

    let registration_state: PasskeyRegistration =
        passkey::from_json_value(state_value)?;

    let credential: RegisterPublicKeyCredential =
        serde_json::from_value(input.credential.clone()).map_err(|err| {
            AppError::Validation(vec![ErrorDetail {
                field: "credential".to_string(),
                message: format!("invalid passkey credential payload: {err}"),
            }])
        })?;

    let passkey = state
        .passkey
        .finish_registration(&credential, &registration_state)?;

    let credential_id = passkey.cred_id().as_ref().to_vec();

    let credential_save = repo::save_user_passkey_credential(
        &state.db,
        auth.user.id,
        credential_id.as_slice(),
        passkey::to_json_value(&passkey)?,
        0,
        challenge
            .challenge_blob
            .get("nickname")
            .and_then(|value| value.as_str()),
    )
    .await?;

    if credential_save
        == repo::PasskeyCredentialSaveStatus::OwnedByDifferentUser
    {
        return Err(AppError::Unauthorized(
            "passkey credential already belongs to another account".to_string(),
        ));
    }

    repo::insert_auth_event(
        &state.db,
        Some(auth.user.id),
        "passkey.register",
        true,
        None,
        None,
        json!({}),
    )
    .await?;

    Ok(auth.user.clone())
}

pub async fn start_passkey_signup(
    state: &AppState,
    headers: &HeaderMap,
    input: PasskeySignupStartRequest,
) -> Result<PasskeyChallengeResponse, AppError> {
    crate::auth::rate_limit::check_auth_rate_limit(
        state,
        "passkey_signup_start",
        headers,
    )
    .await?;

    let email = normalize_email(&input.email)?;
    let display_name =
        normalize_display_name(input.display_name.as_deref(), &email)?;

    if repo::find_user_by_email(&state.db, &email).await?.is_some() {
        return Err(validation_error(
            "email",
            "an account with this email already exists",
        ));
    }

    let provisional_user_id = Uuid::new_v4();

    let (options, state_blob) = state.passkey.start_registration(
        provisional_user_id,
        &email,
        &display_name,
        &[],
    )?;

    let expires_at = OffsetDateTime::now_utc() + Duration::minutes(5);
    let challenge_id = repo::create_webauthn_challenge(
        &state.db,
        None,
        "register",
        json!({
            "state": passkey::to_json_value(&state_blob)?,
            "nickname": input.nickname,
            "signup": PendingPasskeySignup {
                user_id: provisional_user_id,
                email: email.clone(),
                display_name: display_name.clone()
            }
        }),
        expires_at,
    )
    .await?;

    Ok(PasskeyChallengeResponse {
        challenge_id,
        options: passkey::to_json_value(&options)?,
    })
}

pub async fn finish_passkey_signup(
    state: &AppState,
    session_handle: &Session,
    headers: &HeaderMap,
    input: PasskeySignupFinishRequest,
) -> Result<AuthUserProfile, AppError> {
    crate::auth::rate_limit::check_auth_rate_limit(
        state,
        "passkey_signup_finish",
        headers,
    )
    .await?;

    let challenge = repo::consume_webauthn_challenge(
        &state.db,
        input.challenge_id,
        "register",
    )
    .await?
    .ok_or_else(|| {
        AppError::Unauthorized(
            "invalid or expired passkey challenge".to_string(),
        )
    })?;

    if challenge.user_id.is_some() {
        return Err(AppError::Unauthorized(
            "invalid passkey signup challenge".to_string(),
        ));
    }

    let state_value = challenge
        .challenge_blob
        .get("state")
        .cloned()
        .ok_or_else(|| {
            AppError::Internal("passkey challenge state missing".to_string())
        })?;

    let signup_value = challenge
        .challenge_blob
        .get("signup")
        .cloned()
        .ok_or_else(|| {
            AppError::Internal(
                "passkey signup challenge payload missing".to_string(),
            )
        })?;

    let registration_state: PasskeyRegistration =
        passkey::from_json_value(state_value)?;
    let signup_payload: PendingPasskeySignup =
        passkey::from_json_value(signup_value)?;

    let credential: RegisterPublicKeyCredential =
        serde_json::from_value(input.credential.clone()).map_err(|err| {
            AppError::Validation(vec![ErrorDetail {
                field: "credential".to_string(),
                message: format!("invalid passkey credential payload: {err}"),
            }])
        })?;

    let passkey = state
        .passkey
        .finish_registration(&credential, &registration_state)?;

    let credential_id = passkey.cred_id().as_ref().to_vec();

    let user = repo::create_user_with_passkey(
        &state.db,
        signup_payload.user_id,
        &signup_payload.email,
        &signup_payload.display_name,
        credential_id.as_slice(),
        passkey::to_json_value(&passkey)?,
        0,
        challenge
            .challenge_blob
            .get("nickname")
            .and_then(|value| value.as_str()),
    )
    .await?;

    let security = repo::ensure_user_auth_security(&state.db, user.id).await?;

    let _ = session::establish_session(
        session_handle,
        user.id,
        AuthMethod::Passkey,
        security.session_version,
    )
    .await?;

    repo::update_last_login(&state.db, user.id).await?;
    repo::mark_authentication_success(&state.db, user.id, "passkey").await?;
    repo::insert_auth_event(
        &state.db,
        Some(user.id),
        "signup.passkey",
        true,
        read_ip(headers),
        read_user_agent(headers),
        json!({}),
    )
    .await?;

    Ok(user)
}

pub async fn start_passkey_login(
    state: &AppState,
    headers: &HeaderMap,
    _input: PasskeyLoginStartRequest,
) -> Result<PasskeyChallengeResponse, AppError> {
    crate::auth::rate_limit::check_auth_rate_limit(
        state,
        "passkey_login_start",
        headers,
    )
    .await?;

    let passkeys = load_all_active_passkeys(state).await?;

    if passkeys.is_empty() {
        return Err(AppError::Unauthorized(
            "no passkey registered for account".to_string(),
        ));
    }

    let (options, auth_state) =
        state.passkey.start_authentication(&passkeys)?;

    let expires_at = OffsetDateTime::now_utc() + Duration::minutes(5);
    let challenge_id = repo::create_webauthn_challenge(
        &state.db,
        None,
        "authenticate",
        json!({
            "state": passkey::to_json_value(&auth_state)?
        }),
        expires_at,
    )
    .await?;

    Ok(PasskeyChallengeResponse {
        challenge_id,
        options: passkey::to_json_value(&options)?,
    })
}

pub async fn finish_passkey_login(
    state: &AppState,
    session_handle: &Session,
    headers: &HeaderMap,
    input: PasskeyLoginFinishRequest,
) -> Result<AuthUserProfile, AppError> {
    crate::auth::rate_limit::check_auth_rate_limit(
        state,
        "passkey_login_finish",
        headers,
    )
    .await?;

    let challenge = repo::consume_webauthn_challenge(
        &state.db,
        input.challenge_id,
        "authenticate",
    )
    .await?
    .ok_or_else(|| {
        AppError::Unauthorized(
            "invalid or expired passkey challenge".to_string(),
        )
    })?;

    let state_value = challenge
        .challenge_blob
        .get("state")
        .cloned()
        .ok_or_else(|| {
            AppError::Internal("passkey auth state missing".to_string())
        })?;
    let auth_state: PasskeyAuthentication =
        passkey::from_json_value(state_value)?;

    let credential: PublicKeyCredential =
        serde_json::from_value(input.credential.clone()).map_err(|err| {
            AppError::Validation(vec![ErrorDetail {
                field: "credential".to_string(),
                message: format!("invalid passkey assertion payload: {err}"),
            }])
        })?;

    let auth_result = state
        .passkey
        .finish_authentication(&credential, &auth_state)?;

    let credential_id = auth_result.cred_id().as_ref().to_vec();

    let row = repo::find_passkey_by_credential_id(
        &state.db,
        credential_id.as_slice(),
    )
    .await?
    .ok_or_else(|| {
        AppError::Unauthorized("passkey credential not recognized".to_string())
    })?;

    if let Some(challenge_user_id) = challenge.user_id {
        if row.user_id != challenge_user_id {
            return Err(AppError::Unauthorized(
                "passkey credential user mismatch".to_string(),
            ));
        }
    }

    let mut stored_passkey: Passkey =
        passkey::from_json_value(row.credential_json.clone())?;
    let _ = stored_passkey.update_credential(&auth_result);

    let user = repo::get_user_by_id(&state.db, row.user_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("user not found".to_string()))?;

    if !user.is_active {
        return Err(AppError::Unauthorized("account is disabled".to_string()));
    }

    let security = repo::ensure_user_auth_security(&state.db, user.id).await?;
    if security.require_reauth {
        return Err(AppError::Unauthorized(
            "additional verification required".to_string(),
        ));
    }
    if security.passkey_login_disabled {
        return Err(AppError::Unauthorized(
            "passkey sign-in is disabled for this account".to_string(),
        ));
    }
    if security
        .locked_until
        .is_some_and(|locked_until| locked_until > OffsetDateTime::now_utc())
    {
        return Err(AppError::Unauthorized(
            "passkey sign-in is temporarily locked".to_string(),
        ));
    }

    let next_sign_count = i64::from(auth_result.counter());

    repo::update_passkey_usage(
        &state.db,
        credential_id.as_slice(),
        passkey::to_json_value(&stored_passkey)?,
        next_sign_count,
    )
    .await?;
    repo::update_last_login(&state.db, user.id).await?;

    let _ = session::establish_session(
        session_handle,
        user.id,
        AuthMethod::Passkey,
        security.session_version,
    )
    .await?;

    repo::mark_authentication_success(&state.db, user.id, "passkey").await?;

    repo::insert_auth_event(
        &state.db,
        Some(user.id),
        "login.passkey",
        true,
        read_ip(headers),
        read_user_agent(headers),
        json!({}),
    )
    .await?;

    Ok(user)
}

async fn load_user_passkeys(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<Passkey>, AppError> {
    let rows = repo::list_user_passkey_credentials(&state.db, user_id).await?;

    rows.into_iter()
        .map(passkey::from_json_value::<Passkey>)
        .collect::<Result<Vec<_>, _>>()
}

async fn load_all_active_passkeys(
    state: &AppState,
) -> Result<Vec<Passkey>, AppError> {
    let rows = repo::list_active_passkey_credentials(&state.db).await?;

    rows.into_iter()
        .map(passkey::from_json_value::<Passkey>)
        .collect::<Result<Vec<_>, _>>()
}

fn normalize_email(raw: &str) -> Result<String, AppError> {
    let email = raw.trim().to_lowercase();
    if email.is_empty() {
        return Err(validation_error("email", "email is required"));
    }
    if !email.contains('@') {
        return Err(validation_error("email", "email must be valid"));
    }
    Ok(email)
}

fn normalize_display_name(
    raw: Option<&str>,
    email: &str,
) -> Result<String, AppError> {
    let fallback = email.split('@').next().unwrap_or("user");
    let candidate = raw.map(str::trim).filter(|value| !value.is_empty());
    let display_name = candidate.unwrap_or(fallback).to_string();

    if display_name.len() > 120 {
        return Err(validation_error(
            "display_name",
            "display_name is too long",
        ));
    }

    Ok(display_name)
}

fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < 12 {
        return Err(validation_error(
            "password",
            "password must be at least 12 characters",
        ));
    }

    if password.len() > 128 {
        return Err(validation_error(
            "password",
            "password must be at most 128 characters",
        ));
    }

    Ok(())
}

async fn record_login_failure(
    state: &AppState,
    headers: &HeaderMap,
    email: &str,
    reason: &str,
) {
    let _ = repo::insert_auth_event(
        &state.db,
        None,
        "login.password",
        false,
        read_ip(headers),
        read_user_agent(headers),
        json!({ "email": email, "reason": reason }),
    )
    .await;
}

fn read_ip(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|raw| raw.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn read_user_agent(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
}
