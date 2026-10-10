use axum::{routing::get, routing::post, Router};

use crate::{features::auth::handlers, state::AppState};

/// Endpoints reachable without a session.
pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/auth/csrf", get(handlers::issue_csrf))
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
}

/// Endpoints mounted behind the authentication middleware.
pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/auth/logout", post(handlers::logout))
        .route("/auth/password", post(handlers::change_password))
        .route("/auth/session", get(handlers::current_session))
}
