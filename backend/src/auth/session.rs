use std::sync::Arc;

use sqlx::PgPool;
use time::Duration;
use tokio::task::JoinHandle;
use tower_sessions::{
    Expiry, Session, SessionManagerLayer, session_store::ExpiredDeletion,
};
use tower_sessions_sqlx_store::PostgresStore;

use crate::{
    auth::types::{AuthMethod, SessionUser},
    error::AppError,
};

pub const SESSION_USER_KEY: &str = "session_user";

#[derive(Clone)]
pub struct SessionRuntime {
    pub layer: SessionManagerLayer<PostgresStore>,
    _cleanup_handle: Arc<JoinHandle<()>>,
}

pub async fn init_session_runtime(
    pool: PgPool,
    cookie_secure: bool,
    cookie_name: &str,
    idle_minutes: i64,
) -> Result<SessionRuntime, AppError> {
    let store = PostgresStore::new(pool);
    store.migrate().await.map_err(|err| {
        AppError::Internal(format!("failed to migrate session store: {err}"))
    })?;

    let cleanup_store = store.clone();
    let cleanup_handle = tokio::spawn(async move {
        let _ = cleanup_store
            .continuously_delete_expired(std::time::Duration::from_secs(60))
            .await;
    });

    let layer = SessionManagerLayer::new(store)
        .with_name(cookie_name.to_string())
        .with_secure(cookie_secure)
        .with_same_site(tower_sessions::cookie::SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::minutes(idle_minutes)));

    Ok(SessionRuntime {
        layer,
        _cleanup_handle: Arc::new(cleanup_handle),
    })
}

pub async fn establish_session(
    session: &Session,
    user_id: uuid::Uuid,
    auth_method: AuthMethod,
    session_version: i32,
) -> Result<SessionUser, AppError> {
    let payload = SessionUser {
        user_id,
        auth_method,
        session_version,
        issued_at: time::OffsetDateTime::now_utc(),
    };

    session.cycle_id().await.map_err(to_session_error)?;
    session
        .insert(SESSION_USER_KEY, &payload)
        .await
        .map_err(to_session_error)?;

    Ok(payload)
}

pub async fn load_session_user(
    session: &Session,
) -> Result<Option<SessionUser>, AppError> {
    session
        .get(SESSION_USER_KEY)
        .await
        .map_err(to_session_error)
}

pub async fn clear_session(session: &Session) -> Result<(), AppError> {
    session.delete().await.map_err(to_session_error)?;
    Ok(())
}

pub fn session_id(session: &Session) -> Option<String> {
    session.id().map(|id| id.to_string())
}

fn to_session_error(error: tower_sessions::session::Error) -> AppError {
    AppError::Internal(format!("session error: {error}"))
}
