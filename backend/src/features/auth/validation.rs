pub use crate::shared::validation::{ValidationError, Validator};

/// Credentials must not be blank before any database work happens.
pub fn validate_login(email: &str, password: &str) -> Result<(), ValidationError> {
    Validator::new()
        .check(!email.trim().is_empty(), "email", "is required")
        .check(!password.is_empty(), "password", "is required")
        .into_result()
}
