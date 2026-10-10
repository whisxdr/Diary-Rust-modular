use axum::{extract::Extension, extract::Query, extract::State, Json};
use serde::Deserialize;

use crate::{
    features::search::{models::SearchHit, repository},
    middleware::authentication::AuthUser,
    shared::{
        errors::AppResult,
        pagination::{Page, PaginationQuery},
    },
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub q: String,
}

pub async fn search(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthUser>,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let (limit, offset) = query.pagination.normalized()?;
    let term = query.q.trim();

    // An empty query returns an empty page instead of matching every row.
    if term.is_empty() {
        let empty: Page<SearchHit> = Page::new(Vec::new(), 0, limit, offset);
        return Ok(Json(serde_json::to_value(empty)?));
    }

    let (hits, total) = repository::search(&state.pool, caller.id, term, limit, offset).await?;
    Ok(Json(serde_json::to_value(Page::new(
        hits, total, limit, offset,
    ))?))
}
