use axum::{routing::patch, Router};

use crate::{features::tags::handlers, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/tags",
            axum::routing::get(handlers::list).post(handlers::create),
        )
        .route(
            "/tags/{id}",
            patch(handlers::update).delete(handlers::delete),
        )
}
