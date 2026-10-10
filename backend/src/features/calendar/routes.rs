use axum::{routing::get, Router};

use crate::{features::calendar::handlers, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/calendar/month", get(handlers::month))
        .route("/calendar/years", get(handlers::years))
}
