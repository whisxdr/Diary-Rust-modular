use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{infrastructure::session, shared::errors::AppError, state::AppState};

/// Identity of the caller, inserted into request extensions by [`require_user`].
#[derive(Debug, Clone, Copy)]
pub struct AuthUser {
    pub id: Uuid,
    pub session_id: Uuid,
}

impl AuthUser {
    pub fn new(id: Uuid, session_id: Uuid) -> Self {
        Self { id, session_id }
    }
}

/// Rejects requests without a live session and hands the identity to handlers.
pub async fn require_user(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    // The cookie header is copied out before any await: `Request` is not `Sync`,
    // so borrowing it across an await would make this future non-`Send`.
    let cookie = request
        .headers()
        .get(axum::http::header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    match authenticate(&state, cookie.as_deref()).await {
        Ok(user) => {
            request.extensions_mut().insert(user);
            next.run(request).await
        }
        Err(error) => error.into_response(),
    }
}

/// Hashes the session cookie and confirms the session is still live.
pub async fn authenticate(
    state: &AppState,
    cookie_header: Option<&str>,
) -> Result<AuthUser, AppError> {
    let token = session::read_cookie(cookie_header, &state.config.session_cookie_name)
        .ok_or_else(|| AppError::Unauthorized("Sign in to continue.".into()))?;

    crate::features::auth::service::resolve_session(state, &token).await
}
