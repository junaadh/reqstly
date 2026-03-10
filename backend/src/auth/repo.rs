use serde_json::Value;
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    auth::types::{AuthSecurityState, AuthUserProfile},
    error::{AppError, ErrorDetail},
};

#[derive(Debug, Clone, FromRow)]
pub struct PasswordIdentityRow {
    pub user_id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub failed_attempts: i32,
    pub locked_until: Option<OffsetDateTime>,
    pub display_name: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, FromRow)]
struct AuthUserRow {
    id: Uuid,
    email: Option<String>,
    display_name: String,
    is_active: bool,
    email_verified: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct WebauthnChallengeRow {
    pub user_id: Option<Uuid>,
    pub challenge_blob: Value,
}

#[derive(Debug, Clone, FromRow)]
pub struct PasskeyCredentialRow {
    pub user_id: Uuid,
    pub credential_json: Value,
    pub sign_count: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct PasskeySummaryRow {
    pub id: Uuid,
    pub nickname: Option<String>,
    pub created_at: OffsetDateTime,
    pub first_used_at: Option<OffsetDateTime>,
    pub last_used_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, FromRow)]
pub struct PasskeyStatsRow {
    pub passkey_count: i64,
    pub first_registered_at: Option<OffsetDateTime>,
    pub first_used_at: Option<OffsetDateTime>,
    pub last_used_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, FromRow)]
struct AuthSecurityRow {
    session_version: i32,
    require_reauth: bool,
    password_login_disabled: bool,
    passkey_login_disabled: bool,
    risk_score: i16,
    compromised_at: Option<OffsetDateTime>,
    locked_until: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasskeyCredentialSaveStatus {
    Saved,
    OwnedByDifferentUser,
}

impl From<AuthUserRow> for AuthUserProfile {
    fn from(value: AuthUserRow) -> Self {
        Self {
            id: value.id,
            email: value.email.unwrap_or_default(),
            display_name: value.display_name,
            is_active: value.is_active,
            email_verified: value.email_verified,
        }
    }
}

impl From<AuthSecurityRow> for AuthSecurityState {
    fn from(value: AuthSecurityRow) -> Self {
        Self {
            session_version: value.session_version,
            require_reauth: value.require_reauth,
            password_login_disabled: value.password_login_disabled,
            passkey_login_disabled: value.passkey_login_disabled,
            risk_score: value.risk_score,
            compromised_at: value.compromised_at,
            locked_until: value.locked_until,
        }
    }
}

pub async fn create_user_with_password(
    pool: &PgPool,
    email: &str,
    display_name: &str,
    password_hash: &str,
) -> Result<AuthUserProfile, AppError> {
    let mut tx = pool.begin().await?;

    let user = sqlx::query_as::<_, AuthUserRow>(
        "INSERT INTO app.app_users (email, display_name)
         VALUES ($1, $2)
         RETURNING id, email, display_name, is_active, email_verified",
    )
    .bind(email)
    .bind(display_name)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_create_user_error)?;

    sqlx::query(
        "INSERT INTO app.user_password_identities (user_id, email, password_hash, last_password_change_at)
         VALUES ($1, $2, $3, NOW())",
    )
    .bind(user.id)
    .bind(email)
    .bind(password_hash)
    .execute(&mut *tx)
    .await
    .map_err(map_create_user_error)?;

    tx.commit().await?;

    Ok(user.into())
}

pub struct CreateUserWithPasskeyInput<'a> {
    pub user_id: Uuid,
    pub email: &'a str,
    pub display_name: &'a str,
    pub credential_id: &'a [u8],
    pub credential_json: Value,
    pub sign_count: i64,
    pub nickname: Option<&'a str>,
}

pub async fn create_user_with_passkey(
    pool: &PgPool,
    input: CreateUserWithPasskeyInput<'_>,
) -> Result<AuthUserProfile, AppError> {
    let mut tx = pool.begin().await?;

    let user = sqlx::query_as::<_, AuthUserRow>(
        "INSERT INTO app.app_users (id, email, display_name)
         VALUES ($1, $2, $3)
         RETURNING id, email, display_name, is_active, email_verified",
    )
    .bind(input.user_id)
    .bind(input.email)
    .bind(input.display_name)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_create_user_error)?;

    let result = sqlx::query(
        "INSERT INTO app.user_passkey_credentials
            (user_id, credential_id, public_key, credential_json, sign_count, nickname)
         VALUES ($1, $2, NULL, $3, $4, $5)
         ON CONFLICT (credential_id)
         DO UPDATE SET
           credential_json = EXCLUDED.credential_json,
           sign_count = EXCLUDED.sign_count,
           nickname = COALESCE(EXCLUDED.nickname, app.user_passkey_credentials.nickname),
           revoked_at = NULL,
           revoked_reason = NULL
         WHERE app.user_passkey_credentials.user_id = EXCLUDED.user_id",
    )
    .bind(user.id)
    .bind(input.credential_id)
    .bind(input.credential_json)
    .bind(input.sign_count)
    .bind(input.nickname)
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Validation(vec![ErrorDetail {
            field: "credential".to_string(),
            message: "passkey credential already belongs to another account"
                .to_string(),
        }]));
    }

    tx.commit().await?;
    Ok(user.into())
}

pub async fn find_password_identity_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<PasswordIdentityRow>, AppError> {
    sqlx::query_as::<_, PasswordIdentityRow>(
        "SELECT
            identities.user_id,
            identities.email,
            identities.password_hash,
            identities.failed_attempts,
            identities.locked_until,
            users.display_name,
            users.is_active,
            users.email_verified,
            users.deleted_at
         FROM app.user_password_identities identities
         JOIN app.app_users users ON users.id = identities.user_id
         WHERE lower(identities.email) = lower($1)
           AND users.deleted_at IS NULL",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)
}

pub async fn get_user_by_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<AuthUserProfile>, AppError> {
    sqlx::query_as::<_, AuthUserRow>(
        "SELECT id, email, display_name, is_active, email_verified
         FROM app.app_users
         WHERE id = $1
           AND deleted_at IS NULL",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map(|opt| opt.map(Into::into))
    .map_err(AppError::from)
}

pub async fn find_user_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<AuthUserProfile>, AppError> {
    sqlx::query_as::<_, AuthUserRow>(
        "SELECT id, email, display_name, is_active, email_verified
         FROM app.app_users
         WHERE lower(email) = lower($1)
           AND deleted_at IS NULL",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .map(|opt| opt.map(Into::into))
    .map_err(AppError::from)
}

pub async fn ensure_user_auth_security(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<AuthSecurityState, AppError> {
    sqlx::query(
        "INSERT INTO app.user_auth_security (user_id)
         VALUES ($1)
         ON CONFLICT (user_id) DO NOTHING",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    // Promote runtime controls from risk/compliance signals while preserving
    // explicit admin-enforced restrictions.
    sqlx::query(
        "UPDATE app.user_auth_security
         SET
           require_reauth = (
             require_reauth
             OR compromised_at IS NOT NULL
             OR risk_score >= 80
           ),
           password_login_disabled = (
             password_login_disabled
             OR compromised_at IS NOT NULL
             OR risk_score >= 90
           ),
           passkey_login_disabled = (
             passkey_login_disabled
             OR compromised_at IS NOT NULL
             OR risk_score >= 95
           )
         WHERE user_id = $1",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    let row = sqlx::query_as::<_, AuthSecurityRow>(
        "SELECT
            session_version,
            require_reauth,
            password_login_disabled,
            passkey_login_disabled,
            risk_score,
            compromised_at,
            locked_until
         FROM app.user_auth_security
         WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        AppError::Internal("auth security state missing for user".to_string())
    })?;

    Ok(row.into())
}

pub async fn update_last_login(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE app.app_users
         SET last_login_at = NOW(), updated_at = NOW()
         WHERE id = $1",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_authentication_success(
    pool: &PgPool,
    user_id: Uuid,
    method: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO app.user_auth_security (user_id)
         VALUES ($1)
         ON CONFLICT (user_id) DO NOTHING",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    sqlx::query(
        "UPDATE app.user_auth_security
         SET
           last_authn_at = NOW(),
           last_password_login_at = CASE
             WHEN $2 = 'password' THEN NOW()
             ELSE last_password_login_at
           END,
           last_passkey_login_at = CASE
             WHEN $2 = 'passkey' THEN NOW()
             ELSE last_passkey_login_at
           END
         WHERE user_id = $1",
    )
    .bind(user_id)
    .bind(method)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn bump_user_session_version(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<i32, AppError> {
    sqlx::query(
        "INSERT INTO app.user_auth_security (user_id)
         VALUES ($1)
         ON CONFLICT (user_id) DO NOTHING",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    sqlx::query_scalar::<_, i32>(
        "UPDATE app.user_auth_security
         SET session_version = session_version + 1,
             require_reauth = FALSE
         WHERE user_id = $1
         RETURNING session_version",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn record_password_login_failure(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE app.user_password_identities
         SET failed_attempts = failed_attempts + 1,
             last_failed_at = NOW(),
             locked_until = CASE
               WHEN failed_attempts + 1 >= 10
                 THEN GREATEST(
                   COALESCE(locked_until, NOW()),
                   NOW() + INTERVAL '15 minutes'
                 )
               ELSE locked_until
             END
         WHERE user_id = $1",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn clear_password_login_failures(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE app.user_password_identities
         SET failed_attempts = 0,
             last_failed_at = NULL,
             locked_until = NULL
         WHERE user_id = $1",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn insert_auth_event(
    pool: &PgPool,
    user_id: Option<Uuid>,
    event_type: &str,
    success: bool,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    metadata: Value,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO app.auth_events
            (user_id, event_type, success, ip_address, user_agent, metadata)
         VALUES ($1, $2, $3, $4::inet, $5, $6)",
    )
    .bind(user_id)
    .bind(event_type)
    .bind(success)
    .bind(ip_address)
    .bind(user_agent)
    .bind(metadata)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn create_ws_token_issuance(
    pool: &PgPool,
    user_id: Uuid,
    token_fingerprint: &[u8],
    expires_at: OffsetDateTime,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    metadata: Value,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO app.ws_token_issuances
           (user_id, token_fingerprint, expires_at, ip_address, user_agent, metadata)
         VALUES ($1, $2, $3, $4::inet, $5, $6)",
    )
    .bind(user_id)
    .bind(token_fingerprint)
    .bind(expires_at)
    .bind(ip_address)
    .bind(user_agent)
    .bind(metadata)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn is_ws_token_issuance_active(
    pool: &PgPool,
    user_id: Uuid,
    token_fingerprint: &[u8],
) -> Result<bool, AppError> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
           SELECT 1
           FROM app.ws_token_issuances
           WHERE user_id = $1
             AND token_fingerprint = $2
             AND revoked_at IS NULL
             AND expires_at >= NOW()
         )",
    )
    .bind(user_id)
    .bind(token_fingerprint)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn mark_ws_token_used(
    pool: &PgPool,
    user_id: Uuid,
    token_fingerprint: &[u8],
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE app.ws_token_issuances
         SET last_used_at = NOW()
         WHERE user_id = $1
           AND token_fingerprint = $2
           AND revoked_at IS NULL",
    )
    .bind(user_id)
    .bind(token_fingerprint)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn revoke_ws_tokens_for_user(
    pool: &PgPool,
    user_id: Uuid,
    reason: &str,
) -> Result<u64, AppError> {
    let result = sqlx::query(
        "UPDATE app.ws_token_issuances
         SET
           revoked_at = NOW(),
           metadata = metadata || jsonb_build_object('revoked_reason', $2)
         WHERE user_id = $1
           AND revoked_at IS NULL
           AND expires_at >= NOW()",
    )
    .bind(user_id)
    .bind(reason)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn create_csrf_token(
    pool: &PgPool,
    session_id: &str,
    user_id: Uuid,
    purpose: &str,
    token_hash: &[u8],
    expires_at: OffsetDateTime,
    metadata: Value,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO app.csrf_tokens
           (session_id, user_id, purpose, token_hash, expires_at, metadata)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(purpose)
    .bind(token_hash)
    .bind(expires_at)
    .bind(metadata)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn is_csrf_token_valid(
    pool: &PgPool,
    session_id: &str,
    user_id: Uuid,
    purpose: &str,
    token_hash: &[u8],
) -> Result<bool, AppError> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
           SELECT 1
           FROM app.csrf_tokens
           WHERE session_id = $1
             AND user_id = $2
             AND purpose = $3
             AND token_hash = $4
             AND consumed_at IS NULL
             AND expires_at >= NOW()
         )",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(purpose)
    .bind(token_hash)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn create_webauthn_challenge(
    pool: &PgPool,
    user_id: Option<Uuid>,
    flow_type: &str,
    challenge_blob: Value,
    expires_at: OffsetDateTime,
) -> Result<Uuid, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO app.webauthn_challenges (user_id, flow_type, challenge_blob, expires_at)
         VALUES ($1, $2, $3, $4)
         RETURNING id",
    )
    .bind(user_id)
    .bind(flow_type)
    .bind(challenge_blob)
    .bind(expires_at)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn consume_webauthn_challenge(
    pool: &PgPool,
    challenge_id: Uuid,
    expected_flow_type: &str,
) -> Result<Option<WebauthnChallengeRow>, AppError> {
    sqlx::query_as::<_, WebauthnChallengeRow>(
        "UPDATE app.webauthn_challenges
         SET consumed_at = NOW()
         WHERE id = $1
           AND flow_type = $2
           AND consumed_at IS NULL
           AND expires_at >= NOW()
         RETURNING user_id, challenge_blob",
    )
    .bind(challenge_id)
    .bind(expected_flow_type)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)
}

pub async fn list_user_passkey_credentials(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<Value>, AppError> {
    sqlx::query_scalar::<_, Value>(
        "SELECT credential_json
         FROM app.user_passkey_credentials
         WHERE user_id = $1
           AND revoked_at IS NULL
         ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub async fn list_active_passkey_credentials(
    pool: &PgPool,
) -> Result<Vec<Value>, AppError> {
    sqlx::query_scalar::<_, Value>(
        "SELECT credentials.credential_json
         FROM app.user_passkey_credentials credentials
         JOIN app.app_users users ON users.id = credentials.user_id
         WHERE users.is_active = TRUE
           AND users.deleted_at IS NULL
           AND credentials.revoked_at IS NULL
         ORDER BY credentials.created_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub async fn list_user_passkey_summaries(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<PasskeySummaryRow>, AppError> {
    sqlx::query_as::<_, PasskeySummaryRow>(
        "SELECT id, nickname, created_at, first_used_at, last_used_at
         FROM app.user_passkey_credentials
         WHERE user_id = $1
           AND revoked_at IS NULL
         ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub async fn get_user_passkey_stats(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<PasskeyStatsRow, AppError> {
    sqlx::query_as::<_, PasskeyStatsRow>(
        "SELECT
            COALESCE(stats.active_passkey_count, 0)::bigint AS passkey_count,
            stats.first_registered_at,
            stats.first_used_at,
            stats.last_used_at
         FROM app.app_users users
         LEFT JOIN app.user_passkey_stats stats ON stats.user_id = users.id
         WHERE users.id = $1
           AND users.deleted_at IS NULL",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("user not found".to_string()))
}

pub async fn save_user_passkey_credential(
    pool: &PgPool,
    user_id: Uuid,
    credential_id: &[u8],
    credential_json: Value,
    sign_count: i64,
    nickname: Option<&str>,
) -> Result<PasskeyCredentialSaveStatus, AppError> {
    let result = sqlx::query(
        "INSERT INTO app.user_passkey_credentials
            (user_id, credential_id, public_key, credential_json, sign_count, nickname)
         VALUES ($1, $2, NULL, $3, $4, $5)
         ON CONFLICT (credential_id)
         DO UPDATE SET
           credential_json = EXCLUDED.credential_json,
           sign_count = EXCLUDED.sign_count,
           nickname = COALESCE(EXCLUDED.nickname, app.user_passkey_credentials.nickname),
           revoked_at = NULL,
           revoked_reason = NULL
         WHERE app.user_passkey_credentials.user_id = EXCLUDED.user_id",
    )
    .bind(user_id)
    .bind(credential_id)
    .bind(credential_json)
    .bind(sign_count)
    .bind(nickname)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(PasskeyCredentialSaveStatus::OwnedByDifferentUser);
    }

    Ok(PasskeyCredentialSaveStatus::Saved)
}

pub async fn find_passkey_by_credential_id(
    pool: &PgPool,
    credential_id: &[u8],
) -> Result<Option<PasskeyCredentialRow>, AppError> {
    sqlx::query_as::<_, PasskeyCredentialRow>(
        "SELECT user_id, credential_json, sign_count
         FROM app.user_passkey_credentials
         WHERE credential_id = $1
           AND revoked_at IS NULL",
    )
    .bind(credential_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)
}

pub async fn update_passkey_usage(
    pool: &PgPool,
    credential_id: &[u8],
    credential_json: Value,
    sign_count: i64,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE app.user_passkey_credentials
         SET credential_json = $2,
             sign_count = $3,
             first_used_at = COALESCE(first_used_at, NOW()),
             last_used_at = NOW()
         WHERE credential_id = $1",
    )
    .bind(credential_id)
    .bind(credential_json)
    .bind(sign_count)
    .execute(pool)
    .await?;

    Ok(())
}

fn map_create_user_error(error: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_error) = &error
        && let Some(constraint) = db_error.constraint()
        && (constraint == "idx_app_users_email_ci_unique"
            || constraint == "idx_user_password_identities_email_ci_unique")
    {
        return AppError::Validation(vec![ErrorDetail {
            field: "email".to_string(),
            message: "an account with this email already exists".to_string(),
        }]);
    }

    AppError::Database(error)
}
