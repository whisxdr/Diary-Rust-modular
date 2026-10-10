use axum::{routing::get, Router};

use crate::{features::exports::handlers, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/exports/archive", get(handlers::download))
}
