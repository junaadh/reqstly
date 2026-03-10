use crate::error::{AppError, ErrorDetail};

pub fn validation_error(field: &str, message: &str) -> AppError {
    AppError::Validation(vec![ErrorDetail {
        field: field.to_string(),
        message: message.to_string(),
    }])
}
