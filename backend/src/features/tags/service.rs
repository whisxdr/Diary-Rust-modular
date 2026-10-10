use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    features::tags::{models::Tag, repository},
    middleware::authentication::AuthUser,
    shared::{
        errors::{AppError, AppResult},
        validation::Validator,
    },
    state::AppState,
};

pub async fn list(state: &AppState, caller: AuthUser) -> AppResult<Vec<Tag>> {
    let rows = repository::list_with_counts(&state.pool, caller.id).await?;

    Ok(rows
        .into_iter()
        .map(|row| Tag {
            id: row.id,
            name: row.name,
            color: row.color,
            entry_count: row.entry_count,
        })
        .collect())
}

pub async fn create(
    state: &AppState,
    caller: AuthUser,
    name: &str,
    color: Option<&str>,
) -> AppResult<Tag> {
    validate(name)?;
    let row = repository::create(&state.pool, caller.id, name, color).await?;
    Ok(Tag {
        id: row.id,
        name: row.name,
        color: row.color,
        entry_count: 0,
    })
}

pub async fn update(
    state: &AppState,
    caller: AuthUser,
    id: Uuid,
    name: Option<&str>,
    color: Option<&str>,
) -> AppResult<Tag> {
    if let Some(name) = name {
        validate(name)?;
    }

    let row = repository::update(&state.pool, caller.id, id, name, color)
        .await?
        .ok_or_else(|| AppError::NotFound("Tag not found.".into()))?;

    Ok(Tag {
        id: row.id,
        name: row.name,
        color: row.color,
        entry_count: 0,
    })
}

pub async fn delete(state: &AppState, caller: AuthUser, id: Uuid) -> AppResult<()> {
    if repository::delete(&state.pool, caller.id, id).await? {
        return Ok(());
    }
    Err(AppError::NotFound("Tag not found.".into()))
}

/// Cross-feature interface used by the entries feature: names to tag ids,
/// creating any tag that does not exist yet.
pub async fn resolve_by_names(
    pool: &PgPool,
    user_id: Uuid,
    names: &[String],
) -> AppResult<Vec<Uuid>> {
    let mut ids = Vec::with_capacity(names.len());

    for name in names {
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        ids.push(
            repository::find_or_create(pool, user_id, name, None)
                .await?
                .id,
        );
    }

    Ok(ids)
}

/// Cross-feature interface used by the entries list filter.
pub async fn find_id(pool: &PgPool, user_id: Uuid, name: &str) -> AppResult<Option<Uuid>> {
    Ok(repository::find_by_name(pool, user_id, name)
        .await?
        .map(|tag| tag.id))
}

fn validate(name: &str) -> AppResult<()> {
    Validator::new()
        .require("name", name, 1, 40)
        .into_result()
        .map_err(AppError::from)
}
