// use crate::config::AzureAd;
// use crate::error::AppError;
// use crate::models::User;
// use openidconnect::{
//     AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
//     RedirectUrl, Scope, TokenResponse,
//     core::{CoreClient, CoreProviderMetadata},
//     reqwest::{self, async_http_client},
// };
// use sqlx::PgPool;
// use url::Url;

// /// Azure AD authorization URL with state token
// pub struct AuthorizationUrlResult {
//     pub url: Url,
//     pub state: CsrfToken,
//     pub nonce: Nonce,
// }

// /// Generate Azure AD authorization URL
// /// Returns the URL to redirect the user to, along with state/nonce for verification
// pub fn generate_authorization_url(
//     config: &AzureAd,
//     redirect_uri: &str,
// ) -> Result<AuthorizationUrlResult, AppError> {
//     let client_id = ClientId::new(config.client_id.clone());
//     let client_secret = ClientSecret::new(config.client_secret.clone());
//     let redirect_url =
//         RedirectUrl::new(redirect_uri.to_string()).map_err(|e| {
//             AppError::Internal(format!("Invalid redirect URL: {e}"))
//         })?;

//     // Create Azure AD client
//     let issuer_url = IssuerUrl::new(format!(
//         "https://login.microsoftonline.com/{}",
//         config.tenant_id
//     ))
//     .map_err(|e| AppError::Internal(format!("Invalid issuer URL: {e}")))?;

//     let provider_metadata =
//         CoreProviderMetadata::discover(&issuer_url, reqwest::http_client)
//             .map_err(|e| {
//                 AppError::Internal(format!(
//                     "Invalid Azure Provider Metadata: {e}"
//                 ))
//             })?;

//     // Build the OpenID Connect client
//     let client = CoreClient::from_provider_metadata(
//         provider_metadata,
//         client_id,
//         Some(client_secret),
//     )
//     .set_redirect_uri(redirect_url);

//     // Generate CSRF token and nonce
//     let state = CsrfToken::new_random();
//     let nonce = Nonce::new_random();

//     // Generate authorization URL
//     let auth_url = client
//         .authorize_url(
//             openidconnect::core::CoreAuthenticationFlow::AuthorizationCode,
//             move || state,
//             move || nonce,
//         )
//         .add_scope(Scope::new("openid".to_string()))
//         .add_scope(Scope::new("email".to_string()))
//         .add_scope(Scope::new("profile".to_string()))
//         .url();

//     Ok(AuthorizationUrlResult {
//         url: auth_url.0,
//         state: auth_url.1,
//         nonce: auth_url.2,
//     })
// }

// /// Exchange authorization code for tokens and validate JWT
// /// Returns the user (created or updated)
// pub async fn exchange_code_for_token_and_get_user(
//     config: &AzureAd,
//     code: &str,
//     state: CsrfToken,
//     nonce: Nonce,
//     redirect_uri: &str,
//     pool: &PgPool,
// ) -> Result<(User, String), AppError> {
//     let client_id = ClientId::new(config.client_id.clone());
//     let client_secret = ClientSecret::new(config.client_secret.clone());
//     let redirect_url =
//         RedirectUrl::new(redirect_uri.to_string()).map_err(|e| {
//             AppError::Internal(format!("Invalid redirect URL: {}", e))
//         })?;

//     let issuer_url = IssuerUrl::new(format!(
//         "https://login.microsoftonline.com/{}",
//         config.tenant_id
//     ))
//     .map_err(|e| AppError::Internal(format!("Invalid issuer URL: {e}")))?;

//     let provider_metadata =
//         CoreProviderMetadata::discover(&issuer_url, reqwest::http_client)
//             .map_err(|e| {
//                 AppError::Internal(format!(
//                     "Invalid Azure Provider Metadata: {e}"
//                 ))
//             })?;

//     // Create Azure AD client
//     let client = CoreClient::from_provider_metadata(
//         provider_metadata,
//         client_id,
//         Some(client_secret),
//     )
//     .set_redirect_uri(redirect_url);

//     // Exchange code for token
//     let code = AuthorizationCode::new(code.to_string());
//     let token_response = client
//         .exchange_code(code)
//         .request_async(async_http_client)
//         .await
//         .map_err(|e| {
//             AppError::Internal(format!(
//                 "Failed to exchange code for token: {}",
//                 e
//             ))
//         })?;

//     // Extract ID token
//     let id_token = token_response.id_token().ok_or_else(|| {
//         AppError::Internal("No ID token in response".to_string())
//     })?;

//     // Verify ID token claims
//     let claims = id_token
//         .claims(&client.id_token_verifier(), &nonce)
//         // claims(&client.id_token_verifier())
//         .map_err(|e| {
//             AppError::Internal(format!(
//                 "Failed to verify ID token claims: {}",
//                 e
//             ))
//         })?;

//     if let Some(expected_token_hash) =

//     // Verify nonce
//     if claims
//         .nonce()
//         .unwrap_or(&Nonce::new("".to_string()))
//         .secret()
//         != nonce
//     {
//         return Err(AppError::Unauthorized("Invalid nonce".to_string()));
//     }

//     // Extract user information
//     let email = claims
//         .email()
//         .ok_or_else(|| AppError::Internal("No email in ID token".to_string()))?
//         .to_string();

//     let name = claims.name().unwrap_or(&email).to_string();

//     // Azure AD subject identifier
//     let subject = claims.subject().identifier();

//     // Create or update user from Azure AD
//     let user = User::create_from_azure(pool, subject, &email, &name).await?;

//     // Create session
//     let (session, token) =
//         crate::models::Session::create(pool, user.id).await?;

//     Ok((user, token))
// }

// /// Validate Azure AD JWT token and get user
// /// This is used for validating tokens from Azure AD (not session tokens)
// pub async fn validate_azure_jwt(
//     jwt: &str,
//     config: &AzureAd,
//     pool: &PgPool,
// ) -> Result<User, AppError> {
//     // Parse the JWT without full validation (simplified)
//     // In production, you'd verify the signature against Azure AD's public keys
//     let parts: Vec<&str> = jwt.split('.').collect();
//     if parts.len() != 3 {
//         return Err(AppError::Unauthorized("Invalid token format".to_string()));
//     }

//     // Decode payload (base64url)
//     let payload = parts[1];
//     let payload = base64_url_decode(payload)?;

//     // Parse as JSON to extract subject
//     let claims: AzureJwtClaims =
//         serde_json::from_str(&payload).map_err(|_| {
//             AppError::Unauthorized("Invalid token payload".to_string())
//         })?;

//     // Find user by Azure AD subject
//     let user = User::find_by_azure_subject(pool, &claims.sub)
//         .await?
//         .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;

//     Ok(user)
// }

// #[derive(serde::Deserialize)]
// struct AzureJwtClaims {
//     sub: String,
//     email: Option<String>,
//     name: Option<String>,
// }

// fn base64_url_decode(input: &str) -> Result<String, AppError> {
//     use base64::prelude::*;

//     // Add padding if needed
//     let mut input = input.to_string();
//     while input.len() % 4 != 0 {
//         input.push('=');
//     }

//     let decoded = BASE64_URL_SAFE_NO_PAD.decode(input).map_err(|_| {
//         AppError::Unauthorized("Invalid base64 encoding".to_string())
//     })?;

//     String::from_utf8(decoded).map_err(|_| {
//         AppError::Unauthorized("Invalid UTF-8 in token".to_string())
//     })
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     #[ignore]
//     fn test_generate_authorization_url() {
//         let config = AzureAd {
//             client_id: "test_client_id".to_string(),
//             tenant_id: "test_tenant_id".to_string(),
//             client_secret: "test_secret".to_string(),
//         };

//         let result = generate_authorization_url(
//             &config,
//             "http://localhost:3000/auth/azure/callback",
//         )
//         .unwrap();

//         assert!(result.url.as_str().contains("login.microsoftonline.com"));
//         assert!(!result.state.secret().is_empty());
//         assert!(!result.nonce.secret().is_empty());
//     }
// }
