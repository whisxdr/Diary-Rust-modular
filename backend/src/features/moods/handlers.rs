use axum::{extract::State, Json};

use crate::{features::moods::service, shared::errors::AppResult, state::AppState};

pub async fn list(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::to_value(
        service::list(&state.pool).await?,
    )?))
}
