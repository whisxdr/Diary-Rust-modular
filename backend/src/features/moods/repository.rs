use sqlx::PgPool;
use uuid::Uuid;

use crate::{features::moods::models::Mood, shared::errors::AppResult};

const COLUMNS: &str = "id, slug, label, icon, score, position";

pub async fn list(pool: &PgPool) -> AppResult<Vec<Mood>> {
    let sql = format!("SELECT {COLUMNS} FROM moods ORDER BY position");
    Ok(sqlx::query_as::<_, Mood>(&sql).fetch_all(pool).await?)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<Mood>> {
    let sql = format!("SELECT {COLUMNS} FROM moods WHERE id = $1");
    Ok(sqlx::query_as::<_, Mood>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

/// Import matches moods by slug, which survives a device switch where the
/// generated ids differ.
pub async fn find_by_slug(pool: &PgPool, slug: &str) -> AppResult<Option<Mood>> {
    let sql = format!("SELECT {COLUMNS} FROM moods WHERE slug = $1");
    Ok(sqlx::query_as::<_, Mood>(&sql)
        .bind(slug)
        .fetch_optional(pool)
        .await?)
}

pub async fn ids_for_slugs(pool: &PgPool, slugs: &[String]) -> AppResult<Vec<(String, Uuid)>> {
    if slugs.is_empty() {
        return Ok(Vec::new());
    }
    Ok(
        sqlx::query_as::<_, (String, Uuid)>("SELECT slug, id FROM moods WHERE slug = ANY($1)")
            .bind(slugs)
            .fetch_all(pool)
            .await?,
    )
}
