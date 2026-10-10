use axum::{
    extract::{Request, State},
    http::{header, Method},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::infrastructure::session;

const HEADER: &str = "x-csrf-token";

/// Double-submit CSRF protection.
///
/// The token lives in a readable cookie and must be echoed in the `X-CSRF-Token`
/// header. A cross-site form post can send the cookie but cannot set a custom
/// header, so those requests fail the check.
pub async fn protect(State(cookie_name): State<String>, request: Request, next: Next) -> Response {
    if is_safe(request.method()) {
        return next.run(request).await;
    }

    let cookie = request
        .headers()
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|raw| session::read_cookie(Some(raw), &cookie_name));

    let provided = request.headers().get(HEADER).and_then(|v| v.to_str().ok());

    match (cookie, provided) {
        (Some(expected), Some(given)) if expected == given => next.run(request).await,
        _ => {
            tracing::warn!("csrf token missing or mismatched");
            (axum::http::StatusCode::FORBIDDEN, "Invalid CSRF token.").into_response()
        }
    }
}

fn is_safe(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_methods_skip_the_check() {
        assert!(is_safe(&Method::GET));
        assert!(is_safe(&Method::HEAD));
        assert!(is_safe(&Method::OPTIONS));
        assert!(!is_safe(&Method::POST));
        assert!(!is_safe(&Method::DELETE));
    }
}
