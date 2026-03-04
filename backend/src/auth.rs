use axum::http::{HeaderMap, header};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub auth_user_id: Uuid,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Claims {
    sub: String,
    email: Option<String>,
}

pub fn authenticate(
    headers: &HeaderMap,
    jwt_secret: &str,
    expected_issuer: &str,
) -> Result<AuthUser, AppError> {
    let authorization = headers
        .get(header::AUTHORIZATION)
        .ok_or_else(|| {
            AppError::Unauthorized("missing Authorization header".to_string())
        })?
        .to_str()
        .map_err(|_| {
            AppError::Unauthorized("invalid Authorization header".to_string())
        })?;

    let token = authorization.strip_prefix("Bearer ").ok_or_else(|| {
        AppError::Unauthorized("expected Bearer token".to_string())
    })?;

    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[expected_issuer]);
    validation.set_audience(&["authenticated"]);

    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|_| {
        AppError::Unauthorized("invalid or expired token".to_string())
    })?;

    let auth_user_id = Uuid::parse_str(&decoded.claims.sub).map_err(|_| {
        AppError::Unauthorized("token subject is not a UUID".to_string())
    })?;

    Ok(AuthUser {
        auth_user_id,
        email: decoded.claims.email,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use jsonwebtoken::{EncodingKey, Header, encode};
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestClaims {
        sub: String,
        email: String,
        aud: String,
        iss: String,
        exp: usize,
    }

    #[test]
    fn authenticate_accepts_valid_token() {
        let secret = "test-secret";
        let issuer = "https://supabase.localhost/auth/v1";
        let sub = Uuid::new_v4();

        let token = encode(
            &Header::default(),
            &TestClaims {
                sub: sub.to_string(),
                email: "qa@example.com".to_string(),
                aud: "authenticated".to_string(),
                iss: issuer.to_string(),
                exp: 9_999_999_999,
            },
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .expect("token should encode");

        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))
                .expect("header should build"),
        );

        let result = authenticate(&headers, secret, issuer)
            .expect("token should authenticate");

        assert_eq!(result.auth_user_id, sub);
        assert_eq!(result.email.as_deref(), Some("qa@example.com"));
    }

    #[test]
    fn authenticate_rejects_missing_header() {
        let headers = HeaderMap::new();
        let err = authenticate(
            &headers,
            "test-secret",
            "https://supabase.localhost/auth/v1",
        )
        .expect_err("missing header should fail");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }

    #[test]
    fn authenticate_rejects_non_uuid_subject() {
        let secret = "test-secret";
        let issuer = "https://supabase.localhost/auth/v1";

        let token = encode(
            &Header::default(),
            &TestClaims {
                sub: "not-a-uuid".to_string(),
                email: "qa@example.com".to_string(),
                aud: "authenticated".to_string(),
                iss: issuer.to_string(),
                exp: 9_999_999_999,
            },
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .expect("token should encode");

        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))
                .expect("header should build"),
        );

        let err = authenticate(&headers, secret, issuer)
            .expect_err("subject invalid");
        assert!(matches!(err, AppError::Unauthorized(_)));
    }
}
