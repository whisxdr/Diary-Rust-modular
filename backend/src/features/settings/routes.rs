use axum::{routing::get, Router};

use crate::{features::settings::handlers, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/settings/preferences",
        get(handlers::get).patch(handlers::update),
    )
}
