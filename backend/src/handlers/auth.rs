// use crate::DbPool;
// use crate::auth::{azure, middleware::AuthenticatedUser, passkey};
// use crate::config::{AzureAd, Passkey};
// use crate::error::AppError;
// use crate::models::Session;
// use axum::{
//     Json,
//     extract::{Query, State},
//     http::StatusCode,
//     response::{IntoResponse, Redirect, Response},
// };
// use serde::Deserialize;
// use tower_cookies::{Cookie, Cookies};

// /// Azure AD callback parameters
// #[derive(Debug, Deserialize)]
// pub struct AzureCallbackParams {
//     pub code: String,
//     pub state: String,
// }

// /// Passkey login start request
// #[derive(Debug, Deserialize)]
// pub struct PasskeyLoginStart {
//     pub email: String,
// }

// /// Azure AD login endpoint
// /// Redirects user to Azure AD for authentication
// pub async fn azure_login(
//     _: State<DbPool>,
//     State(azure_config): State<AzureAd>,
// ) -> Result<Redirect, AppError> {
//     let redirect_uri = "http://0.0.0.0/api/auth/azure/callback"; // TODO: Make configurable

//     let result =
//         azure::generate_authorization_url(&azure_config, redirect_uri)?;

//     // TODO: Store state and nonce in session/redis for verification
//     // For now, we'll rely on Azure's state verification

//     Ok(Redirect::temporary(result.url.as_str()))
// }

// /// Azure AD callback endpoint
// /// Handles the OAuth callback from Azure AD
// pub async fn azure_callback(
//     Query(params): Query<AzureCallbackParams>,
//     State(pool): State<DbPool>,
//     State(azure_config): State<AzureAd>,
//     cookies: Cookies,
// ) -> Result<impl IntoResponse, AppError> {
//     let redirect_uri = "http://localhost/api/auth/azure/callback"; // TODO: Make configurable

//     // Exchange code for token and get user
//     let (_user, token) = azure::exchange_code_for_token_and_get_user(
//         &azure_config,
//         &params.code,
//         &params.state,
//         "", // nonce would come from session
//         redirect_uri,
//         &pool,
//     )
//     .await?;

//     // Set session cookie
//     let mut cookie = Cookie::new("session", token);
//     cookie.set_http_only(true);
//     cookie.set_secure(false); // TODO: true in production
//     cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
//     cookie.set_path("/");
//     cookies.add(cookie);

//     // Redirect to frontend
//     Ok(Redirect::to("/"))
// }

// /// Start passkey authentication
// /// Generates challenge for user to authenticate with passkey
// pub async fn passkey_login_start(
//     Json(payload): Json<PasskeyLoginStart>,
//     State(pool): State<DbPool>,
//     State(passkey_config): State<Passkey>,
// ) -> Result<Json<passkey::PasskeyChallenge>, AppError> {
//     let options =
//         passkey::start_authentication(&payload.email, &pool, &passkey_config)
//             .await?;
//     Ok(Json(options))
// }

// /// Finish passkey authentication
// /// Verifies the passkey assertion and creates session
// pub async fn passkey_login_finish(
//     Json(response): Json<passkey::PasskeyAuthenticationResponse>,
//     State(pool): State<DbPool>,
//     State(passkey_config): State<Passkey>,
//     cookies: Cookies,
// ) -> Result<Response, AppError> {
//     // Verify passkey assertion and get user
//     let user = passkey::finish_authentication(response, &pool, &passkey_config)
//         .await?;

//     // Create session
//     let (_session, token) = Session::create(&pool, user.id).await?;

//     // Set session cookie
//     let mut cookie = Cookie::new("session", token);
//     cookie.set_http_only(true);
//     cookie.set_secure(false); // TODO: true in production
//     cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
//     cookie.set_path("/");
//     cookies.add(cookie);

//     Ok((
//         StatusCode::OK,
//         Json(serde_json::json!({
//             "message": "Authenticated successfully",
//             "user_id": user.id,
//             "email": user.email
//         })),
//     )
//         .into_response())
// }

// /// Start passkey registration (authenticated)
// /// Generates challenge for user to register a new passkey
// pub async fn passkey_register_start(
//     user: AuthenticatedUser,
//     State(pool): State<DbPool>,
//     State(passkey_config): State<Passkey>,
// ) -> Result<Json<passkey::PasskeyChallenge>, AppError> {
//     let options =
//         passkey::start_registration(user.id, &pool, &passkey_config).await?;
//     Ok(Json(options))
// }

// /// Finish passkey registration (authenticated)
// /// Verifies the passkey credential and stores it
// pub async fn passkey_register_finish(
//     user: AuthenticatedUser,
//     Json(response): Json<passkey::PasskeyRegistrationResponse>,
//     State(pool): State<DbPool>,
//     State(passkey_config): State<Passkey>,
// ) -> Result<Response, AppError> {
//     let credential =
//         passkey::finish_registration(user.id, response, &pool, &passkey_config)
//             .await?;

//     Ok((
//         StatusCode::CREATED,
//         Json(serde_json::json!({
//             "message": "Passkey registered successfully",
//             "credential_id": credential.credential_id
//         })),
//     )
//         .into_response())
// }

// /// Logout endpoint
// /// Invalidates the current session
// pub async fn logout(
//     _user: AuthenticatedUser,
//     State(pool): State<DbPool>,
//     cookies: Cookies,
// ) -> Result<Response, AppError> {
//     // Get session cookie
//     let session_cookie = cookies
//         .get("session")
//         .ok_or(AppError::BadRequest("No session cookie found".to_string()))?;

//     // Invalidate session
//     Session::invalidate(&pool, session_cookie.value()).await?;

//     // Remove session cookie
//     let mut cookie = Cookie::from("session");
//     cookie.set_path("/");
//     cookie.set_max_age(tower_cookies::cookie::time::Duration::ZERO);
//     cookies.add(cookie);

//     Ok((
//         StatusCode::OK,
//         Json(serde_json::json!({
//             "message": "Logged out successfully"
//         })),
//     )
//         .into_response())
// }

// /// Get current user info
// pub async fn me(user: AuthenticatedUser) -> impl IntoResponse {
//     Json(serde_json::json!({
//         "id": user.id,
//         "email": user.email,
//         "name": user.name
//     }))
// }
