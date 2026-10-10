use axum::{routing::get, Router};

use crate::{features::moods::handlers, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/moods", get(handlers::list))
}
