use crate::config::Passkey;
use crate::error::AppError;
use crate::models::{PasskeyCredential, User};
use sqlx::PgPool;
use uuid::Uuid;

/// Passkey registration response from client
#[derive(Debug, serde::Deserialize)]
pub struct PasskeyRegistrationResponse {
    pub id: String,
    pub raw_id: String,
    pub response: serde_json::Value,
    pub type_: String,
}

/// Passkey authentication response from client
#[derive(Debug, serde::Deserialize)]
pub struct PasskeyAuthenticationResponse {
    pub id: String,
    pub raw_id: String,
    pub response: serde_json::Value,
    pub type_: String,
}

/// Placeholder challenge response
#[derive(Debug, serde::Serialize)]
pub struct PasskeyChallenge {
    pub challenge: String,
    pub message: String,
}

/// Start passkey registration ceremony
/// Generates challenge for the user to register a passkey
pub async fn start_registration(
    _user_id: Uuid,
    _pool: &PgPool,
    _config: &Passkey,
) -> Result<PasskeyChallenge, AppError> {
    // Stub only
    Err(AppError::Internal(
        "Passkey registration not yet implemented.".to_string(),
    ))
}

/// Finish passkey registration ceremony
/// Verifies the credential and stores it
pub async fn finish_registration(
    _user_id: Uuid,
    _response: PasskeyRegistrationResponse,
    _pool: &PgPool,
    _config: &Passkey,
) -> Result<PasskeyCredential, AppError> {
    // Stub only
    Err(AppError::Internal(
        "Passkey registration not yet implemented.".to_string(),
    ))
}

/// Start passkey authentication ceremony
/// Generates challenge for the user to authenticate with passkey
pub async fn start_authentication(
    _user_email: &str,
    _pool: &PgPool,
    _config: &Passkey,
) -> Result<PasskeyChallenge, AppError> {
    // Stub only
    Err(AppError::Internal(
        "Passkey authentication not yet implemented.".to_string(),
    ))
}

/// Finish passkey authentication ceremony
/// Verifies the assertion and returns the authenticated user
pub async fn finish_authentication(
    _response: PasskeyAuthenticationResponse,
    _pool: &PgPool,
    _config: &Passkey,
) -> Result<User, AppError> {
    // Stub only
    Err(AppError::Internal(
        "Passkey authentication not yet implemented.".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore]
    async fn test_start_registration() {
        // Stub
    }

    #[tokio::test]
    #[ignore]
    async fn test_start_authentication() {
        // Stub
    }
}
