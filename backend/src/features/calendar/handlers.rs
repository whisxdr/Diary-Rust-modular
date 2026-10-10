use axum::{
    extract::{Extension, Query, State},
    Json,
};
use serde::Deserialize;

use crate::{
    features::calendar::service, middleware::authentication::AuthUser, shared::errors::AppResult,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct MonthQuery {
    pub year: i32,
    pub month: u32,
}

pub async fn month(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Query(query): Query<MonthQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let month = service::month(&state, caller, query.year, query.month).await?;
    Ok(Json(serde_json::to_value(month)?))
}

pub async fn years(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::to_value(
        service::years(&state, caller).await?,
    )?))
}
