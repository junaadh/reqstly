use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use openidconnect::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    reqwest::async_http_client,
};
use redis::Commands;
use tower_cookies::{Cookie, Cookies};

use crate::{
    AppState,
    config::AzureAd,
    error::AppError,
    models::{
        external_identities::{AuthProvider, ExternalIdentity},
        session::Session,
        user::User,
    },
};

const REDIS_STATE_TTL_SECS: u64 = 300;

pub async fn store_state_nonce_verifier(
    redis: &redis::Client,
    state: &CsrfToken,
    nonce: &Nonce,
    verifier: &PkceCodeVerifier,
) -> Result<(), AppError> {
    let mut conn = redis.get_connection().map_err(AppError::from)?;
    let key = format!("oidc_state:{}", state.secret());
    let value = serde_json::to_string(&(nonce.secret(), verifier.secret()))
        .map_err(|e| {
            AppError::Internal(format!("Failed to serialize PKCE: {e}"))
        })?;
    conn.set_ex(key, value, REDIS_STATE_TTL_SECS)
        .map_err(AppError::from)
}

pub async fn consume_state_nonce_verifier(
    redis: &redis::Client,
    state: &CsrfToken,
) -> Result<(Nonce, PkceCodeVerifier), AppError> {
    let mut conn = redis.get_connection().map_err(AppError::from)?;
    let key = format!("oidc_state:{}", state.secret());
    let nonce: Option<String> = conn.get(&key).map_err(AppError::from)?;

    match nonce {
        Some(n) => {
            let _: () = conn.del(&key).map_err(AppError::from)?;
            let (nonce_str, verifier_str) =
                serde_json::from_str::<(String, String)>(&n).map_err(|_| {
                    AppError::Unauthorized("Invalid PKCE data".into())
                })?;

            Ok((Nonce::new(nonce_str), PkceCodeVerifier::new(verifier_str)))
        }
        None => Err(AppError::Unauthorized(
            "Invalid or expired state".to_string(),
        )),
    }
}

#[derive(Clone)]
pub struct AzureOidc {
    pub client: CoreClient,
    pub redirect_uri: String,
}

impl AzureOidc {
    pub async fn new(
        config: &AzureAd,
        redirect_uri: String,
    ) -> Result<Self, AppError> {
        let issuer = IssuerUrl::new(format!(
            "https://login.microsoftonline.com/{}/v2.0",
            config.tenant_id
        ))
        .map_err(|err| {
            AppError::Internal(format!(
                "Failed to create azure issuerer: {err}"
            ))
        })?;

        let metadata =
            CoreProviderMetadata::discover_async(issuer, async_http_client)
                .await
                .map_err(|err| AppError::Internal(err.to_string()))?;

        let client = CoreClient::from_provider_metadata(
            metadata,
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_uri.to_string())
                .map_err(|err| AppError::Internal(err.to_string()))?,
        );

        Ok(Self {
            client,
            redirect_uri,
        })
    }
}

pub async fn azure_login(
    State(state): State<AppState>,
) -> Result<Redirect, AppError> {
    let azure = state.azure_client.as_ref().ok_or_else(|| {
        AppError::Unauthorized("Azure AD is not configured".to_string())
    })?;
    let (pkce_challenge, pkce_code_verifier) =
        PkceCodeChallenge::new_random_sha256();

    let (auth_url, state_token, nonce_token) = azure
        .client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .set_pkce_challenge(pkce_challenge)
        .add_scope(Scope::new("openid".into()))
        .add_scope(Scope::new("email".into()))
        .add_scope(Scope::new("profile".into()))
        .url();

    store_state_nonce_verifier(
        &state.redis,
        &state_token,
        &nonce_token,
        &pkce_code_verifier,
    )
    .await?;

    Ok(Redirect::to(auth_url.as_str()))
}

pub async fn azure_callback(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    let azure = state.azure_client.as_ref().ok_or_else(|| {
        AppError::Unauthorized("Azure AD is not configured".to_string())
    })?;
    if let Some(err) = params.get("error") {
        let desc = params
            .get("error_description")
            .map(String::as_str)
            .unwrap_or("no description");

        return Err(AppError::Unauthorized(format!(
            "Azure error: {err} ({desc})"
        )));
    }

    let code = params
        .get("code")
        .ok_or(AppError::Unauthorized("Missing Code".into()))?;
    let state_str = params
        .get("state")
        .ok_or(AppError::Unauthorized("Missing State".into()))?;
    let csrf_token = CsrfToken::new(state_str.to_owned());

    let (nonce, pkce_code_verifier) =
        consume_state_nonce_verifier(&state.redis, &csrf_token).await?;

    let token_response = azure
        .client
        .exchange_code(AuthorizationCode::new(code.to_owned()))
        .set_pkce_verifier(pkce_code_verifier)
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            AppError::Unauthorized(format!("Token exchange failed: {e}"))
        })?;

    let id_token = token_response
        .id_token()
        .ok_or_else(|| AppError::Unauthorized("Missing ID token".to_owned()))?;

    let claims = id_token
        .claims(&azure.client.id_token_verifier(), &nonce)
        .map_err(|e| {
            AppError::Unauthorized(format!("ID token validation failed: {e}"))
        })?;

    let email = claims
        .email()
        .map(|e| e.to_string())
        .or_else(|| claims.preferred_username().map(|u| u.to_string()));

    let name = claims
        .name()
        .and_then(|n| n.get(None))
        .map(|n| n.to_string());

    let subject = claims.subject().to_string();

    let provider = AuthProvider::AzureAd;

    let (user, identity) = match ExternalIdentity::find_by_provider_subject(
        &state.db, provider, &subject,
    )
    .await?
    {
        Some(identity) => {
            let user = User::find_by_id(&state.db, identity.user_id)
                .await?
                .ok_or_else(|| {
                    AppError::Internal(
                        "External identity found but user missing".to_string(),
                    )
                })?;
            (user, Some(identity))
        }
        None => {
            let user = ExternalIdentity::resolve_user_from_external_identity(
                &state.db,
                provider,
                &subject,
                email.as_deref(),
                name.as_deref(),
            )
            .await?;

            let identity = ExternalIdentity::find_by_provider_subject(
                &state.db, provider, &subject,
            )
            .await?;

            (user, identity)
        }
    };

    let identity = identity.ok_or_else(|| {
        AppError::Internal("External identity not created".to_string())
    })?;

    let (_session, token) =
        Session::create(&state.db, user.id, Some(identity), provider).await?;

    let mut cookie = Cookie::new("session", token.into_inner());
    cookie.set_http_only(true);
    cookie.set_secure(false);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
    cookie.set_path("/");
    cookies.add(cookie);

    Ok((
        StatusCode::OK,
        axum::Json(serde_json::json!({
            "user": {
                "id": user.id,
                "email": user.email,
                "name": user.name
            }
        })),
    ))
}
