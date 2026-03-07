use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    Password,
    Passkey,
    Oidc(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUser {
    pub user_id: Uuid,
    pub auth_method: AuthMethod,
    pub session_version: i32,
    pub issued_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PasswordLoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PasskeyRegisterStartRequest {
    pub nickname: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PasskeySignupStartRequest {
    pub email: String,
    pub display_name: Option<String>,
    pub nickname: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PasskeyRegisterFinishRequest {
    pub challenge_id: Uuid,
    pub credential: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PasskeySignupFinishRequest {
    pub challenge_id: Uuid,
    pub credential: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PasskeyLoginStartRequest {}

#[derive(Debug, Clone, Deserialize)]
pub struct PasskeyLoginFinishRequest {
    pub challenge_id: Uuid,
    pub credential: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthUserProfile {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub is_active: bool,
    pub email_verified: bool,
}

#[derive(Debug, Clone)]
pub struct AuthSecurityState {
    pub session_version: i32,
    pub require_reauth: bool,
    pub password_login_disabled: bool,
    pub passkey_login_disabled: bool,
    pub risk_score: i16,
    pub compromised_at: Option<OffsetDateTime>,
    pub locked_until: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WsTokenResponse {
    pub token: String,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct CsrfTokenResponse {
    pub token: String,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct PasskeyChallengeResponse {
    pub challenge_id: Uuid,
    pub options: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct PasskeyCredentialSummary {
    pub id: Uuid,
    pub nickname: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub first_used_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_used_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PasskeyStats {
    pub passkey_count: i64,
    #[serde(with = "time::serde::rfc3339::option")]
    pub first_registered_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub first_used_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_used_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PasskeyListResponse {
    pub credentials: Vec<PasskeyCredentialSummary>,
    pub stats: PasskeyStats,
}

#[cfg(test)]
mod tests {
    use super::PasskeyCredentialSummary;
    use serde_json::Value;
    use time::OffsetDateTime;
    use uuid::Uuid;

    #[test]
    fn passkey_summary_serializes_rfc3339_timestamps() {
        let summary = PasskeyCredentialSummary {
            id: Uuid::new_v4(),
            nickname: Some("Device Key".to_string()),
            created_at: OffsetDateTime::UNIX_EPOCH,
            first_used_at: Some(OffsetDateTime::UNIX_EPOCH),
            last_used_at: Some(OffsetDateTime::UNIX_EPOCH),
        };

        let value =
            serde_json::to_value(summary).expect("summary should serialize");

        let created_at = value
            .get("created_at")
            .and_then(Value::as_str)
            .expect("created_at should be a string");
        let first_used_at = value
            .get("first_used_at")
            .and_then(Value::as_str)
            .expect("first_used_at should be a string");
        let last_used_at = value
            .get("last_used_at")
            .and_then(Value::as_str)
            .expect("last_used_at should be a string");

        assert!(created_at.contains('T'));
        assert!(first_used_at.contains('T'));
        assert!(last_used_at.contains('T'));
    }
}
