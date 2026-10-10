use axum::{
    extract::{Extension, State},
    Json,
};

use crate::{
    features::settings::{models::UpdatePreferencesRequest, service},
    middleware::authentication::AuthUser,
    shared::errors::AppResult,
    state::AppState,
};

pub async fn get(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::to_value(
        service::get(&state, caller).await?,
    )?))
}

pub async fn update(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Json(request): Json<UpdatePreferencesRequest>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::to_value(
        service::update(&state, caller, request).await?,
    )?))
}
