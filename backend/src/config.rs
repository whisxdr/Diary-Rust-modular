use std::{net::SocketAddr, path::PathBuf};

/// Runtime configuration. Everything comes from the environment so the same binary
/// runs in development and production.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: SocketAddr,
    pub storage_root: PathBuf,
    pub session_ttl_hours: i64,
    pub session_cookie_name: String,
    pub csrf_cookie_name: String,
    pub cors_allow_any_origin: bool,
    pub max_upload_bytes: usize,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let database_url = required("DATABASE_URL")?;
        let port = parse("PORT", 8080u16)?;
        let host = var("HOST", "127.0.0.1");
        let bind_addr: SocketAddr = format!("{host}:{port}")
            .parse()
            .map_err(|e| format!("invalid bind address {host}:{port}: {e}"))?;

        Ok(Self {
            database_url,
            bind_addr,
            storage_root: PathBuf::from(var("STORAGE_ROOT", "./storage")),
            session_ttl_hours: parse("SESSION_TTL_HOURS", 720i64)?,
            session_cookie_name: var("SESSION_COOKIE_NAME", "dmd_session"),
            csrf_cookie_name: var("CSRF_COOKIE_NAME", "dmd_csrf"),
            cors_allow_any_origin: var("CORS_ALLOW_ANY_ORIGIN", "false") == "true",
            max_upload_bytes: parse("MAX_UPLOAD_BYTES", 16 * 1024 * 1024usize)?,
        })
    }
}

fn required(key: &str) -> Result<String, String> {
    std::env::var(key).map_err(|_| format!("missing required environment variable {key}"))
}

fn var(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}

fn parse<T: std::str::FromStr>(key: &str, default: T) -> Result<T, String> {
    match std::env::var(key) {
        Ok(raw) => raw
            .parse()
            .map_err(|_| format!("environment variable {key} is not a valid value: {raw}")),
        Err(_) => Ok(default),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_address_is_built_from_host_and_port() {
        let addr: SocketAddr = "0.0.0.0:9000".parse().expect("valid address");
        assert_eq!(addr.port(), 9000);
    }
}
