use crate::{
    AppState,
    error::AppError,
    models::{
        Session, User,
        external_identities::AuthProvider,
        password::{CreatePassword, Password, PasswordLogin, PasswordSignup},
    },
};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use serde_json::json;
use tower_cookies::{Cookie, Cookies};

pub fn create_password_routes() -> Router<AppState> {
    Router::new()
        .route("/signup", post(password_signup))
        .route("/login", post(password_login))
}

/// Password signup handler
/// Creates a new user with a password
pub async fn password_signup(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(input): Json<PasswordSignup>,
) -> Result<Response, AppError> {
    // Validate input
    if input.email.is_empty()
        || input.name.is_empty()
        || input.password.is_empty()
    {
        return Err(AppError::BadRequest(
            "Email, name, and password are required".to_string(),
        ));
    }

    // Validate password strength (min 8 characters)
    if input.password.len() < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    // Check if user already exists
    if User::find_by_email(&state.db, &input.email)
        .await?
        .is_some()
    {
        return Err(AppError::BadRequest(
            "User with this email already exists".to_string(),
        ));
    }

    // Create the user
    let user = User::create(
        &state.db,
        crate::models::CreateUser {
            email: input.email.clone(),
            name: input.name.clone(),
        },
    )
    .await?;

    // Create the password (separate table for security)
    Password::create(
        &state.db,
        CreatePassword {
            user_id: user.id,
            password: input.password,
        },
    )
    .await?;

    // Create a session
    let (_, token) =
        Session::create(&state.db, user.id, None, AuthProvider::Password)
            .await?;

    // Set session cookie
    let mut cookie = Cookie::new("session", token.as_ref().to_string());
    cookie.set_path("/");
    cookie.set_secure(true);
    cookie.set_same_site(tower_cookies::cookie::SameSite::None);
    cookies.add(cookie);

    tracing::info!("User created via password signup: {}", user.email);

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "Account created successfully",
            "user": {
                "id": user.id,
                "email": user.email,
                "name": user.name,
            }
        })),
    )
        .into_response())
}

/// Password login handler
/// Authenticates a user with email and password
pub async fn password_login(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(input): Json<PasswordLogin>,
) -> Result<Response, AppError> {
    // Validate input
    if input.email.is_empty() || input.password.is_empty() {
        return Err(AppError::BadRequest(
            "Email and password are required".to_string(),
        ));
    }

    // Find user by email
    let user = User::find_by_email(&state.db, &input.email)
        .await?
        .ok_or_else(|| {
            AppError::Unauthorized("Invalid email or password".to_string())
        })?;

    // Find password for user
    let password = Password::find_by_user_id(&state.db, user.id)
        .await?
        .ok_or_else(|| {
            AppError::Unauthorized("Invalid email or password".to_string())
        })?;

    // Verify password
    if !password.verify(&input.password)? {
        return Err(AppError::Unauthorized(
            "Invalid email or password".to_string(),
        ));
    }

    // Create a session
    let (_, token) =
        Session::create(&state.db, user.id, None, AuthProvider::Password)
            .await?;

    // Set session cookie
    let mut cookie = Cookie::new("session", token.as_ref().to_string());
    cookie.set_path("/");
    cookie.set_secure(true);
    cookie.set_same_site(tower_cookies::cookie::SameSite::None);
    cookies.add(cookie);

    tracing::info!("User logged in via password: {}", user.email);

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Login successful",
            "user": {
                "id": user.id,
                "email": user.email,
                "name": user.name,
            }
        })),
    )
        .into_response())
}
