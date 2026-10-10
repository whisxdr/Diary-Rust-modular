use axum::{
    extract::{Extension, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::{
    features::accounts::{
        dto::DeleteAccountRequest, dto::UpdateProfileRequest, models::Profile, service,
    },
    features::auth::dto::MessageResponse,
    middleware::authentication::AuthUser,
    shared::errors::AppResult,
    state::AppState,
};

pub async fn profile(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::to_value(
        service::get_profile(&state, caller).await?,
    )?))
}

pub async fn update_profile(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Json(request): Json<UpdateProfileRequest>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::to_value(
        service::update_profile(&state, caller, request).await?,
    )?))
}

#[derive(Serialize)]
struct DeletionResponse {
    message: String,
    summary: crate::features::accounts::dto::DeletionSummary,
}

pub async fn delete_account(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Json(request): Json<DeleteAccountRequest>,
) -> AppResult<Response> {
    let summary = service::delete_account(&state, caller, &request.confirm_display_name).await?;

    let cleared = crate::infrastructure::session::clear_cookie(&state.config.session_cookie_name);
    Ok((
        [(axum::http::header::SET_COOKIE, cleared)],
        Json(DeletionResponse {
            message: MessageResponse::new("Account deleted.").message,
            summary,
        }),
    )
        .into_response())
}

/// Type alias kept so the profile shape stays in one place.
pub type AccountProfile = Profile;
