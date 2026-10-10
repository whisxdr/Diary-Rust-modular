use axum::{extract::Extension, extract::State, Json};

use crate::{
    features::insights::service, middleware::authentication::AuthUser, shared::errors::AppResult,
    state::AppState,
};

pub async fn overview(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::to_value(
        service::overview(&state, caller).await?,
    )?))
}
