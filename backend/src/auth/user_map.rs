use tower_sessions::Session;

use crate::{
    AppState,
    auth::{middleware, types::AuthUserProfile},
    error::AppError,
};

pub async fn resolve_current_user(
    state: &AppState,
    session_handle: &Session,
    headers: &axum::http::HeaderMap,
) -> Result<AuthUserProfile, AppError> {
    let context =
        middleware::resolve_request_auth(state, session_handle, headers)
            .await?;
    Ok(context.user)
}
