use axum::{
    http::{HeaderValue, Method},
    middleware,
    routing::get,
    Router,
};
use tower_http::{
    cors::{AllowOrigin, Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    features::{
        accounts, attachments, auth, calendar, entries, exports, imports, insights, moods, search,
        settings, tags,
    },
    middleware::{authentication, csrf, rate_limit},
    state::AppState,
};

pub const API_PREFIX: &str = "/api";

/// Composes the application router. Adding a feature means adding one line here.
pub fn build_router(state: AppState) -> Router {
    let protected = Router::new()
        .merge(auth::routes::protected_router())
        .merge(accounts::routes::router())
        .merge(entries::routes::router())
        .merge(tags::routes::router())
        .merge(moods::routes::router())
        .merge(calendar::routes::router())
        .merge(attachments::routes::router())
        .merge(imports::routes::router())
        .merge(exports::routes::router())
        .merge(search::routes::router())
        .merge(insights::routes::router())
        .merge(settings::routes::router())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            authentication::require_user,
        ));

    Router::new()
        .route("/health", get(health))
        .nest(API_PREFIX, auth::routes::public_router().merge(protected))
        .layer(middleware::from_fn_with_state(
            rate_limit::RateLimiter::new(),
            rate_limit::enforce,
        ))
        .layer(middleware::from_fn_with_state(
            state.config.csrf_cookie_name.clone(),
            csrf::protect,
        ))
        .layer(cors_layer(&state))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

fn cors_layer(state: &AppState) -> CorsLayer {
    let base = if state.config.cors_allow_any_origin {
        CorsLayer::new().allow_origin(Any)
    } else {
        // Same-origin by default: the dev frontend runs on its own port.
        let origin: AllowOrigin = [HeaderValue::from_static("http://localhost:5173")]
            .into_iter()
            .collect::<Vec<_>>()
            .into();
        CorsLayer::new().allow_origin(origin)
    };

    base.allow_methods([
        Method::GET,
        Method::POST,
        Method::PATCH,
        Method::PUT,
        Method::DELETE,
    ])
    .allow_credentials(true)
}
