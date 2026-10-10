use axum::{routing::get, Router};

use crate::{features::insights::handlers, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/insights", get(handlers::overview))
}
