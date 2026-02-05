use axum::{Json, extract::State, http::StatusCode};
use base64::{Engine, engine::general_purpose};
use redis::Commands;
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};
use webauthn_rs::prelude::*;
use webauthn_rs_proto::{
    PublicKeyCredentialCreationOptions, PublicKeyCredentialRequestOptions,
    RegisteredExtensions, UserVerificationPolicy,
};

use crate::{
    AppState,
    auth::session_token::SessionToken,
    error::AppError,
    models::{
        self, external_identities::AuthProvider,
        passkey::CreatePasskeyCredential,
    },
};

pub async fn store_session_passkey_reg_uid(
    redis: &redis::Client,
    token: &SessionToken,
    reg_state: &PasskeyRegistration,
    user_id: &Uuid,
) -> Result<(), AppError> {
    let mut conn = redis.get_connection().map_err(AppError::from)?;
    let key = format!("passkey:register:{}", token.as_ref());
    let value = serde_json::to_string(&(reg_state, user_id)).map_err(|e| {
        AppError::Internal(format!("Failed to serialize reg_state: {e}"))
    })?;
    conn.set_ex(key, value, 600).map_err(AppError::from)
}

pub async fn consume_session_passkey_reg_uid(
    redis: &redis::Client,
    token: &SessionToken,
) -> Result<(PasskeyRegistration, Uuid), AppError> {
    let mut conn = redis.get_connection().map_err(AppError::from)?;
    let key = format!("passkey:register:{}", token.as_ref());
    let nonce: Option<String> = conn.get(&key).map_err(AppError::from)?;

    match nonce {
        Some(n) => {
            let _: () = conn.del(&key).map_err(AppError::from)?;
            let reg_state =
                serde_json::from_str::<(PasskeyRegistration, Uuid)>(&n)
                    .map_err(|_| {
                        AppError::Unauthorized("Invalid reg_state data".into())
                    })?;

            Ok(reg_state)
        }
        None => Err(AppError::Unauthorized(
            "Invalid or expired state".to_string(),
        )),
    }
}

#[derive(Deserialize)]
pub struct PasskeyRegisterStartRequest {
    pub name: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct PasskeyLoginStartRequest {
    pub email: String,
}

pub async fn passkey_register_start(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(req): Json<PasskeyRegisterStartRequest>,
) -> Result<Json<PublicKeyCredentialCreationOptions>, AppError> {
    let session_token = cookies
        .get("session")
        .map(|c| SessionToken::new(c.value().to_string()));

    let invalid_or_non_session =
        async || -> Result<(Uuid, SessionToken), AppError> {
            let user = models::User::create(
                &state.db,
                models::CreateUser {
                    email: req.email.clone(),
                    name: req.name.clone(),
                },
            )
            .await?;

            let (_, token) = models::Session::create(
                &state.db,
                user.id,
                None,
                AuthProvider::Passkey,
            )
            .await?;
            cookies.add(
                Cookie::build(("session", token.clone().into_inner()))
                    .http_only(true)
                    .secure(false)
                    .same_site(tower_cookies::cookie::SameSite::Lax)
                    .path("/")
                    .into(),
            );

            Ok((user.id, token))
        };

    let (user_id, session_token) = if let Some(token) = session_token {
        match models::Session::find_valid(&state.db, &token)
            .await
            .ok()
            .flatten()
        {
            Some((s, _)) => (s.user_id, token),
            None => invalid_or_non_session().await?,
        }
    } else {
        invalid_or_non_session().await?
    };

    let (opts, reg_state) = state
        .webauthn
        .start_passkey_registration(user_id, &req.email, &req.name, None)
        .map_err(|e| {
            AppError::Internal(format!(
                "Failed to start passkey registration: {e}"
            ))
        })?;

    store_session_passkey_reg_uid(
        &state.redis,
        &session_token,
        &reg_state,
        &user_id,
    )
    .await?;

    Ok(Json(opts.public_key))
}

pub async fn passkey_register_finish(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(credential): Json<RegisterPublicKeyCredential>,
) -> Result<StatusCode, AppError> {
    let session_token = cookies
        .get("session")
        .ok_or(AppError::Unauthorized("Missing session".to_owned()))
        .map(|x| SessionToken::new(x.value().to_string()))?;

    let (reg_state, user_id) =
        consume_session_passkey_reg_uid(&state.redis, &session_token).await?;

    let result = state
        .webauthn
        .finish_passkey_registration(&credential, &reg_state)
        .map_err(|e| {
            AppError::Unauthorized(format!("Passkey registration failed: {e}"))
        })?;

    let cred = CreatePasskeyCredential {
        user_id,
        credential_id: general_purpose::URL_SAFE_NO_PAD
            .encode(result.cred_id()),
        public_key: serde_json::to_string(result.get_public_key()).map_err(
            |e| {
                AppError::Internal(format!(
                    "failed to serialize public key: {e}"
                ))
            },
        )?,
        transports: Some(Vec::new()),
    };

    models::PasskeyCredential::create(&state.db, cred).await?;

    Ok(StatusCode::CREATED)
}

// FIXME: credential field on passkey provided by webauthn is private
// pub async fn passkey_login_start(
//     State(state): State<AppState>,
//     cookies: Cookies,
//     Json(req): Json<PasskeyLoginStartRequest>,
// ) -> Result<Json<PublicKeyCredentialRequestOptions>, AppError> {
//     // 1. Look up the user by email
//     let user = models::User::find_by_email(&state.db, &req.email)
//         .await?
//         .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

//     // 2. Get or create session token
//     let session_token = if let Some(cookie) = cookies.get("session") {
//         let token = SessionToken::new(cookie.value().to_string());
//         match models::Session::find_valid(&state.db, &token)
//             .await
//             .ok()
//             .flatten()
//         {
//             Some((s, _)) => token, // reuse valid session
//             None => {
//                 let (_, token) = models::Session::create(
//                     &state.db,
//                     user.id,
//                     None,
//                     AuthProvider::Passkey,
//                 )
//                 .await?;
//                 cookies.add(
//                     Cookie::build(("session", token.clone().into_inner()))
//                         .http_only(true)
//                         .secure(false)
//                         .same_site(tower_cookies::cookie::SameSite::Lax)
//                         .path("/")
//                         .into(),
//                 );
//                 token
//             }
//         }
//     } else {
//         let (_, token) = models::Session::create(
//             &state.db,
//             user.id,
//             None,
//             AuthProvider::Passkey,
//         )
//         .await?;
//         cookies.add(
//             Cookie::build(("session", token.clone().into_inner()))
//                 .http_only(true)
//                 .secure(false)
//                 .same_site(tower_cookies::cookie::SameSite::Lax)
//                 .path("/")
//                 .into(),
//         );
//         token
//     };

//     let passkeys =
//         models::PasskeyCredential::find_by_user_id(&state.db, user.id)
//             .await?
//             .into_iter()
//             .map(|pk| Passkey {
//                 cred: Credential {
//                     cred_id: general_purpose::URL_SAFE_NO_PAD
//                         .decode(pk.credential_id)
//                         .unwrap()
//                         .into(),
//                     cred: serde_json::from_str(&pk.public_key).unwrap(),
//                     counter: pk.counter as u32,
//                     transports: None,
//                     user_verified: false,
//                     backup_eligible: false,
//                     backup_state: false,
//                     registration_policy:
//                         UserVerificationPolicy::Discouraged_DO_NOT_USE,
//                     extensions: RegisteredExtensions::none(),
//                     attestation: ParsedAttestation::default(),
//                     attestation_format: AttestationFormat::None,
//                 },
//             })
//             .collect::<Vec<_>>();

//     // 3. Start WebAuthn login challenge
//     let (opts, login_state) = state
//         .webauthn
//         .start_passkey_authentication(&passkeys)
//         .map_err(|e| {
//             AppError::Internal(format!("Failed to start passkey login: {e}"))
//         })?;

//     // 4. Store challenge in Redis (session-linked)
//     let mut conn = state.redis.get_connection().map_err(AppError::from)?;
//     let key = format!("passkey:login:{}", session_token.as_ref());
//     let value = serde_json::to_string(&login_state).map_err(|e| {
//         AppError::Internal(format!("Failed to serialize login_state: {e}"))
//     })?;
//     conn.set_ex(key, value, 300).map_err(AppError::from)?; // 5 min TTL

//     Ok(Json(opts.public_key))
// }

// TODO: add passkey_login_finish route
