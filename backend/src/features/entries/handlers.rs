use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use crate::{
    features::entries::{dto::*, service},
    middleware::authentication::AuthUser,
    shared::errors::AppResult,
    state::AppState,
};

pub async fn create(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Json(request): Json<CreateEntryRequest>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    let entry = service::create(&state, caller, request).await?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(entry)?)))
}

pub async fn get(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let entry = service::get(&state, caller, id).await?;
    Ok(Json(serde_json::to_value(entry)?))
}

pub async fn get_by_date(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Path(date): Path<String>,
) -> AppResult<Json<Option<serde_json::Value>>> {
    let entry = service::get_by_date(&state, caller, &date).await?;
    Ok(Json(entry.map(serde_json::to_value).transpose()?))
}

pub async fn list(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Query(query): Query<EntryQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let page = service::list(&state, caller, query).await?;
    Ok(Json(serde_json::to_value(page)?))
}

pub async fn update(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateEntryRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let entry = service::update(&state, caller, id, request).await?;
    Ok(Json(serde_json::to_value(entry)?))
}

pub async fn autosave(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<AutosaveRequest>,
) -> AppResult<Response> {
    let entry = service::autosave(&state, caller, id, request).await?;
    Ok(Json(serde_json::to_value(entry)?).into_response())
}

pub async fn set_status(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<StatusRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let entry = service::set_status(&state, caller, id, request).await?;
    Ok(Json(serde_json::to_value(entry)?))
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Response> {
    service::delete(&state, caller, id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
