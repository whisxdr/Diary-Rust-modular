use std::{net::SocketAddr, path::PathBuf};

/// Runtime configuration. Everything comes from the environment so the same binary
/// runs in development, on a single machine, and behind a CDN-hosted frontend.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: SocketAddr,
    pub storage_root: PathBuf,
    pub session_ttl_hours: i64,
    pub session_cookie_name: String,
    pub csrf_cookie_name: String,
    /// `Lax` when the API serves its own frontend, `None` when the frontend is
    /// hosted on another site (a CDN deployment, for example).
    pub cookie_same_site: String,
    /// Browsers only accept `SameSite=None` cookies over HTTPS.
    pub cookie_secure: bool,
    pub cors_allow_any_origin: bool,
    /// Exact origins allowed to call the API.
    pub allowed_origins: Vec<String>,
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

        let cookie_same_site = normalize_same_site(&var("COOKIE_SAME_SITE", "Lax"))?;
        let cookie_secure = match var("COOKIE_SECURE", "auto").as_str() {
            "auto" => cookie_same_site == "None",
            "true" => true,
            "false" => false,
            other => {
                return Err(format!(
                    "COOKIE_SECURE must be auto, true or false, got `{other}`"
                ))
            }
        };

        Ok(Self {
            database_url,
            bind_addr,
            storage_root: PathBuf::from(var("STORAGE_ROOT", "./storage")),
            session_ttl_hours: parse("SESSION_TTL_HOURS", 720i64)?,
            session_cookie_name: var("SESSION_COOKIE_NAME", "dmd_session"),
            csrf_cookie_name: var("CSRF_COOKIE_NAME", "dmd_csrf"),
            cookie_same_site,
            cookie_secure,
            cors_allow_any_origin: var("CORS_ALLOW_ANY_ORIGIN", "false") == "true",
            allowed_origins: parse_origins(&var("ALLOWED_ORIGINS", "http://localhost:5173")),
            max_upload_bytes: parse("MAX_UPLOAD_BYTES", 16 * 1024 * 1024usize)?,
        })
    }
}

fn normalize_same_site(raw: &str) -> Result<String, String> {
    match raw.to_ascii_lowercase().as_str() {
        "lax" => Ok("Lax".to_owned()),
        "strict" => Ok("Strict".to_owned()),
        "none" => Ok("None".to_owned()),
        other => Err(format!(
            "COOKIE_SAME_SITE must be Lax, Strict or None, got `{other}`"
        )),
    }
}

/// Comma separated list. Vectors are joined with commas so both
/// `ALLOWED_ORIGINS=a,b` and `ALLOWED_ORIGINS=["a","b"]` work.
fn parse_origins(raw: &str) -> Vec<String> {
    raw.trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|origin| origin.trim().trim_matches('"').to_owned())
        .filter(|origin| !origin.is_empty())
        .collect()
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
    fn origins_accept_plain_and_json_shapes() {
        assert_eq!(
            parse_origins("https://a.vercel.app, https://b.vercel.app"),
            vec!["https://a.vercel.app", "https://b.vercel.app"]
        );
        assert_eq!(
            parse_origins("[\"https://a.vercel.app\"]"),
            vec!["https://a.vercel.app"]
        );
        assert!(parse_origins("  ").is_empty());
    }

    #[test]
    fn same_site_values_are_normalized_and_validated() {
        assert_eq!(normalize_same_site("none").unwrap(), "None");
        assert_eq!(normalize_same_site("LAX").unwrap(), "Lax");
        assert!(normalize_same_site("whatever").is_err());
    }
}
