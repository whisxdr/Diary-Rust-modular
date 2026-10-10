use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use crate::{
    features::tags::{dto::*, service},
    middleware::authentication::AuthUser,
    shared::errors::AppResult,
    state::AppState,
};

pub async fn list(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::to_value(
        service::list(&state, caller).await?,
    )?))
}

pub async fn create(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Json(request): Json<CreateTagRequest>,
) -> AppResult<Response> {
    let tag = service::create(&state, caller, &request.name, request.color.as_deref()).await?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(tag)?)).into_response())
}

pub async fn update(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateTagRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let tag = service::update(
        &state,
        caller,
        id,
        request.name.as_deref(),
        request.color.as_deref(),
    )
    .await?;
    Ok(Json(serde_json::to_value(tag)?))
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Response> {
    service::delete(&state, caller, id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
