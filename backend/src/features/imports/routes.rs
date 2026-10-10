use axum::{routing::get, routing::post, Router};

use crate::{features::imports::handlers, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/imports", post(handlers::upload))
        .route("/imports/history", get(handlers::history))
        .route("/imports/{job_id}/preview", get(handlers::preview))
        .route("/imports/commit", post(handlers::commit))
}
