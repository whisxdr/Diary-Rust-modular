use axum::{routing::delete, routing::get, Router};

use crate::{features::accounts::handlers, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/account",
            get(handlers::profile).patch(handlers::update_profile),
        )
        .route("/account", delete(handlers::delete_account))
}
