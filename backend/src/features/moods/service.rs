use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    features::{entries::models::MoodSummary, moods::models::Mood, moods::repository},
    shared::errors::AppResult,
};

pub async fn list(pool: &PgPool) -> AppResult<Vec<Mood>> {
    repository::list(pool).await
}

/// Cross-feature interface used by the entries feature.
pub async fn summary(pool: &PgPool, id: Uuid) -> AppResult<Option<MoodSummary>> {
    Ok(repository::find_by_id(pool, id)
        .await?
        .map(|mood| MoodSummary {
            id: mood.id,
            slug: mood.slug,
            label: mood.label,
            icon: mood.icon,
        }))
}

pub async fn id_for_slug(pool: &PgPool, slug: &str) -> AppResult<Option<Uuid>> {
    Ok(repository::find_by_slug(pool, slug)
        .await?
        .map(|mood| mood.id))
}
