use axum::{routing::get, Router};

use crate::{features::search::handlers, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/search", get(handlers::search))
}
