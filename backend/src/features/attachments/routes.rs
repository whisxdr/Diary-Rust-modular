use axum::{routing::delete, routing::get, routing::post, Router};

use crate::{features::attachments::handlers, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/attachments", post(handlers::upload))
        .route("/attachments/{id}/download", get(handlers::download))
        .route("/attachments/{id}", delete(handlers::delete))
        .route(
            "/entries/{entry_id}/attachments",
            get(handlers::list_for_entry),
        )
}
