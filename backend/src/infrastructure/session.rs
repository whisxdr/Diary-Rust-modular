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

/// Cookie attributes come from configuration so the same code serves a
/// same-origin install and a frontend hosted on another site.
#[derive(Debug, Clone, Copy)]
pub struct CookieOptions<'name, 'config> {
    pub name: &'name str,
    pub same_site: &'config str,
    pub secure: bool,
}

impl<'name, 'config> CookieOptions<'name, 'config> {
    pub fn new(name: &'name str, config: &'config crate::config::Config) -> Self {
        Self {
            name,
            same_site: &config.cookie_same_site,
            secure: config.cookie_secure,
        }
    }

    fn attributes(self, value: &str, max_age_secs: i64, http_only: bool) -> String {
        let secure = if self.secure { "; Secure" } else { "" };
        let http_only = if http_only { "; HttpOnly" } else { "" };
        format!(
            "{name}={value}; Path=/; Max-Age={max_age_secs}; SameSite={same_site}{secure}{http_only}",
            name = self.name,
            same_site = self.same_site,
        )
    }
}

/// Reads one cookie from a `Cookie` header value.
pub fn read_cookie(header: Option<&str>, name: &str) -> Option<String> {
    let header = header?;
    header.split(';').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key.trim() == name).then(|| value.trim().to_owned())
    })
}

pub fn build_cookie(options: CookieOptions<'_, '_>, value: &str, max_age_secs: i64) -> String {
    options.attributes(value, max_age_secs, true)
}

/// Readable by JavaScript on purpose: the frontend copies it into a request header.
pub fn build_csrf_cookie(options: CookieOptions<'_, '_>, value: &str, max_age_secs: i64) -> String {
    options.attributes(value, max_age_secs, false)
}

pub fn clear_cookie(options: CookieOptions<'_, '_>) -> String {
    options.attributes("", 0, true)
}

/// Expiry reported to the client for a freshly opened or current session.
pub fn current_expiry(config: &crate::config::Config) -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now() + chrono::Duration::hours(config.session_ttl_hours)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAX: CookieOptions<'static, 'static> = CookieOptions {
        name: "dmd_session",
        same_site: "Lax",
        secure: false,
    };

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

    #[test]
    fn session_cookie_is_httponly_and_csrf_cookie_is_not() {
        let session = build_cookie(LAX, "abc", 60);
        assert!(session.starts_with("dmd_session=abc;"));
        assert!(session.contains("HttpOnly"));
        assert!(session.contains("SameSite=Lax"));
        assert!(!session.contains("Secure"));

        let csrf = build_csrf_cookie(LAX, "abc", 60);
        assert!(csrf.contains("dmd_session=abc;"));
        assert!(!csrf.contains("HttpOnly"));
    }

    #[test]
    fn cross_site_cookies_require_https() {
        let cross_site = CookieOptions {
            name: "dmd_session",
            same_site: "None",
            secure: true,
        };
        let cookie = build_cookie(cross_site, "abc", 60);
        assert!(cookie.contains("SameSite=None"));
        assert!(cookie.contains("Secure"));
    }

    #[test]
    fn clearing_a_cookie_expires_it() {
        assert!(clear_cookie(LAX).contains("Max-Age=0"));
    }
}
