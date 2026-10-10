use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};

use crate::shared::errors::{AppError, ValidationError};

/// Argon2id with the default (OWASP-recommended) parameters. Verification is
/// constant-time inside the crate, so no extra comparison is needed here.
pub fn hash(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Internal(format!("password hashing failed: {e}")))
}

pub fn verify(password: &str, password_hash: &str) -> Result<(), AppError> {
    let parsed = PasswordHash::new(password_hash)
        .map_err(|_| AppError::Internal("stored password hash is unreadable".into()))?;

    match Argon2::default().verify_password(password.as_bytes(), &parsed) {
        Ok(()) => Ok(()),
        Err(_) => Err(AppError::Unauthorized(
            "Email or password is incorrect.".into(),
        )),
    }
}

/// Reports the same error for unknown email and wrong password.
pub fn invalid_credentials() -> AppError {
    AppError::Unauthorized("Email or password is incorrect.".into())
}

pub fn require_str(value: &str, field: &str) -> Result<String, ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::field(field, "is required"));
    }
    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_verify_and_reject() {
        let hash = hash("correct horse battery").expect("hashing works");
        assert!(verify("correct horse battery", &hash).is_ok());
        assert!(verify("wrong password", &hash).is_err());
    }

    #[test]
    fn hashes_are_salted() {
        let a = hash("same password").unwrap();
        let b = hash("same password").unwrap();
        assert_ne!(a, b, "each hash must use a fresh salt");
    }
}
