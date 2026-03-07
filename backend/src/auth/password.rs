use crate::error::AppError;

pub fn hash_password(password: &str) -> Result<String, AppError> {
    Ok(password_auth::generate_hash(password))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    match password_auth::verify_password(password, hash) {
        Ok(_) => Ok(true),
        Err(password_auth::VerifyError::PasswordInvalid) => Ok(false),
        Err(err) => Err(AppError::Internal(format!(
            "failed to verify password: {err}"
        ))),
    }
}
