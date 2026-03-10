use serde::de::DeserializeOwned;
use serde_json::Value;
use url::Url;
use webauthn_rs::prelude::*;

use crate::error::AppError;

#[derive(Clone)]
pub struct PasskeyService {
    webauthn: Webauthn,
}

impl PasskeyService {
    pub fn new(
        rp_id: &str,
        rp_origin: &str,
        rp_name: &str,
    ) -> Result<Self, AppError> {
        let origin = Url::parse(rp_origin).map_err(|err| {
            AppError::Internal(format!("invalid webauthn rp origin: {err}"))
        })?;

        let builder = WebauthnBuilder::new(rp_id, &origin).map_err(|err| {
            AppError::Internal(format!(
                "invalid webauthn builder config: {err}"
            ))
        })?;

        let webauthn = builder.rp_name(rp_name).build().map_err(|err| {
            AppError::Internal(format!(
                "failed to build webauthn service: {err}"
            ))
        })?;

        Ok(Self { webauthn })
    }

    pub fn start_registration(
        &self,
        user_id: uuid::Uuid,
        username: &str,
        display_name: &str,
        existing_credentials: &[Passkey],
    ) -> Result<(CreationChallengeResponse, PasskeyRegistration), AppError>
    {
        let excluded: Vec<CredentialID> = existing_credentials
            .iter()
            .map(|credential| credential.cred_id().clone())
            .collect();

        self.webauthn
            .start_passkey_registration(
                user_id,
                username,
                display_name,
                Some(excluded),
            )
            .map_err(|err| {
                AppError::Internal(format!(
                    "failed to start passkey registration: {err}"
                ))
            })
    }

    pub fn finish_registration(
        &self,
        credential: &RegisterPublicKeyCredential,
        state: &PasskeyRegistration,
    ) -> Result<Passkey, AppError> {
        self.webauthn
            .finish_passkey_registration(credential, state)
            .map_err(|err| {
                AppError::Unauthorized(format!(
                    "passkey registration verification failed: {err}"
                ))
            })
    }

    pub fn start_authentication(
        &self,
        passkeys: &[Passkey],
    ) -> Result<(RequestChallengeResponse, PasskeyAuthentication), AppError>
    {
        self.webauthn
            .start_passkey_authentication(passkeys)
            .map_err(|err| {
                AppError::Internal(format!(
                    "failed to start passkey authentication: {err}"
                ))
            })
    }

    pub fn finish_authentication(
        &self,
        credential: &PublicKeyCredential,
        state: &PasskeyAuthentication,
    ) -> Result<AuthenticationResult, AppError> {
        self.webauthn
            .finish_passkey_authentication(credential, state)
            .map_err(|err| {
                AppError::Unauthorized(format!(
                    "passkey authentication verification failed: {err}"
                ))
            })
    }
}

pub fn to_json_value<T: serde::Serialize>(
    value: &T,
) -> Result<Value, AppError> {
    serde_json::to_value(value).map_err(|err| {
        AppError::Internal(format!("failed to serialize passkey state: {err}"))
    })
}

pub fn from_json_value<T: DeserializeOwned>(
    value: Value,
) -> Result<T, AppError> {
    serde_json::from_value(value).map_err(|err| {
        AppError::Internal(format!(
            "failed to deserialize passkey state: {err}"
        ))
    })
}

pub fn decode_credential_id_from_json(
    credential: &Value,
) -> Result<Vec<u8>, AppError> {
    use base64::Engine;

    let id = credential
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            AppError::Validation(vec![crate::error::ErrorDetail {
                field: "credential.id".to_string(),
                message: "credential id is required".to_string(),
            }])
        })?;

    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(id)
        .map_err(|_| {
            AppError::Validation(vec![crate::error::ErrorDetail {
                field: "credential.id".to_string(),
                message: "credential id must be base64url".to_string(),
            }])
        })
}
