use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};

/// Generates an opaque session token. Only its hash is stored in the database,
/// so a database leak cannot be replayed as a login.
pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn generate_csrf_token() -> String {
    generate_token()
}

/// Reads one cookie from a `Cookie` header value.
pub fn read_cookie(header: Option<&str>, name: &str) -> Option<String> {
    let header = header?;
    header.split(';').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key.trim() == name).then(|| value.trim().to_owned())
    })
}

pub fn build_cookie(name: &str, value: &str, max_age_secs: i64) -> String {
    format!("{name}={value}; Path=/; Max-Age={max_age_secs}; HttpOnly; SameSite=Lax")
}

/// Readable by JavaScript on purpose: the frontend copies it into a request header.
pub fn build_csrf_cookie(name: &str, value: &str, max_age_secs: i64) -> String {
    format!("{name}={value}; Path=/; Max-Age={max_age_secs}; SameSite=Lax")
}

pub fn clear_cookie(name: &str) -> String {
    format!("{name}=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax")
}

/// Expiry reported to the client for a freshly opened or current session.
pub fn current_expiry(config: &crate::config::Config) -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now() + chrono::Duration::hours(config.session_ttl_hours)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_lookup_ignores_unrelated_names() {
        let header = Some("other=1; dmd_session=abc; extra=2");
        assert_eq!(read_cookie(header, "dmd_session").as_deref(), Some("abc"));
        assert_eq!(read_cookie(header, "missing"), None);
    }

    #[test]
    fn tokens_are_unique_and_hash_stably() {
        let a = generate_token();
        let b = generate_token();
        assert_ne!(a, b);
        assert_eq!(hash_token(&a), hash_token(&a));
        assert_ne!(hash_token(&a), hash_token(&b));
    }
}
