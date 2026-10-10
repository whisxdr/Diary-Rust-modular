use axum::{
    routing::{get, patch, post},
    Router,
};

use crate::{features::entries::handlers, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/entries", post(handlers::create).get(handlers::list))
        .route("/entries/by-date/{date}", get(handlers::get_by_date))
        .route(
            "/entries/{id}",
            get(handlers::get)
                .patch(handlers::update)
                .delete(handlers::delete),
        )
        .route("/entries/{id}/autosave", patch(handlers::autosave))
        .route("/entries/{id}/status", patch(handlers::set_status))
}
