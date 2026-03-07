use axum::http::HeaderMap;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::FromRow;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{AppState, error::AppError};

#[derive(Debug, Clone, Copy)]
struct RateLimitPolicy {
    max_attempts: i32,
    window_seconds: i32,
    block_seconds: i32,
}

#[derive(Debug, FromRow)]
struct RateLimitBucketRow {
    id: Uuid,
    attempt_count: i32,
    blocked_until: Option<OffsetDateTime>,
}

pub async fn check_auth_rate_limit(
    state: &AppState,
    scope: &str,
    headers: &HeaderMap,
) -> Result<(), AppError> {
    let policy = policy_for(scope);
    let now = OffsetDateTime::now_utc();
    let window_started_at = to_window_start(now, policy.window_seconds)?;
    let key_hash = rate_limit_key_hash(scope, headers);

    let row = sqlx::query_as::<_, RateLimitBucketRow>(
        "INSERT INTO app.auth_rate_limit_buckets
           (scope, key_hash, window_started_at, window_seconds, attempt_count, last_attempt_at, metadata)
         VALUES ($1, $2, $3, $4, 1, $5, $6)
         ON CONFLICT (scope, key_hash, window_started_at, window_seconds)
         DO UPDATE SET
           attempt_count = app.auth_rate_limit_buckets.attempt_count + 1,
           last_attempt_at = EXCLUDED.last_attempt_at
         RETURNING id, attempt_count, blocked_until",
    )
    .bind(scope)
    .bind(&key_hash)
    .bind(window_started_at)
    .bind(policy.window_seconds)
    .bind(now)
    .bind(json!({
      "scope": scope,
      "ip": read_ip(headers),
      "has_user_agent": read_user_agent(headers).is_some()
    }))
    .fetch_one(&state.db)
    .await?;

    if row
        .blocked_until
        .is_some_and(|blocked_until| blocked_until > now)
    {
        return Err(AppError::RateLimited(
            "too many authentication attempts; try again later".to_string(),
        ));
    }

    if row.attempt_count > policy.max_attempts {
        let blocked_until =
            now + Duration::seconds(i64::from(policy.block_seconds));
        sqlx::query(
            "UPDATE app.auth_rate_limit_buckets
             SET blocked_until = GREATEST(COALESCE(blocked_until, NOW()), $2)
             WHERE id = $1",
        )
        .bind(row.id)
        .bind(blocked_until)
        .execute(&state.db)
        .await?;

        return Err(AppError::RateLimited(
            "too many authentication attempts; try again later".to_string(),
        ));
    }

    Ok(())
}

fn policy_for(scope: &str) -> RateLimitPolicy {
    match scope {
        "signup" => RateLimitPolicy {
            max_attempts: 6,
            window_seconds: 900,
            block_seconds: 900,
        },
        "login_password" => RateLimitPolicy {
            max_attempts: 10,
            window_seconds: 900,
            block_seconds: 900,
        },
        "passkey_signup_start" | "passkey_signup_finish" => RateLimitPolicy {
            max_attempts: 10,
            window_seconds: 900,
            block_seconds: 900,
        },
        "passkey_login_start" | "passkey_login_finish" => RateLimitPolicy {
            max_attempts: 15,
            window_seconds: 900,
            block_seconds: 600,
        },
        "passkey_register_start" | "passkey_register_finish" => {
            RateLimitPolicy {
                max_attempts: 30,
                window_seconds: 900,
                block_seconds: 300,
            }
        }
        _ => RateLimitPolicy {
            max_attempts: 30,
            window_seconds: 300,
            block_seconds: 300,
        },
    }
}

fn to_window_start(
    timestamp: OffsetDateTime,
    window_seconds: i32,
) -> Result<OffsetDateTime, AppError> {
    let now_ts = timestamp.unix_timestamp();
    let width = i64::from(window_seconds);
    let window_start_ts = now_ts - now_ts.rem_euclid(width);
    OffsetDateTime::from_unix_timestamp(window_start_ts).map_err(|_| {
        AppError::Internal("failed to derive rate-limit window".to_string())
    })
}

fn rate_limit_key_hash(scope: &str, headers: &HeaderMap) -> Vec<u8> {
    let ip = read_ip(headers).unwrap_or("unknown");
    let user_agent = read_user_agent(headers).unwrap_or("unknown");
    let material = format!("{scope}|{ip}|{user_agent}");
    Sha256::digest(material.as_bytes()).to_vec()
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
