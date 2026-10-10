use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::{
    features::tags::models::{TagRow, TagWithCount},
    shared::errors::AppResult,
};

/// Tags of one account with the number of entries using each.
pub async fn list_with_counts(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<TagWithCount>> {
    let sql = "SELECT t.id, t.user_id, t.name, t.color, t.created_at,
                      count(et.entry_id) AS entry_count
               FROM tags t
               LEFT JOIN entry_tags et ON et.tag_id = t.id
               WHERE t.user_id = $1
               GROUP BY t.id
               ORDER BY t.normalized";

    Ok(sqlx::query_as::<_, TagWithCount>(sql)
        .bind(user_id)
        .fetch_all(pool)
        .await?)
}

pub async fn find_by_id(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<Option<TagRow>> {
    let sql =
        "SELECT id, user_id, name, color, created_at FROM tags WHERE id = $1 AND user_id = $2";
    Ok(sqlx::query_as::<_, TagRow>(sql)
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?)
}

/// Case-insensitive lookup; the generated `normalized` column does the matching.
pub async fn find_by_name(pool: &PgPool, user_id: Uuid, name: &str) -> AppResult<Option<TagRow>> {
    let sql = "SELECT id, user_id, name, color, created_at FROM tags
               WHERE user_id = $1 AND normalized = $2";
    Ok(sqlx::query_as::<_, TagRow>(sql)
        .bind(user_id)
        .bind(name.trim().to_lowercase())
        .fetch_optional(pool)
        .await?)
}

/// Returns the existing tag or inserts it. Used by the editor and by import.
pub async fn find_or_create(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    color: Option<&str>,
) -> AppResult<TagRow> {
    if let Some(existing) = find_by_name(pool, user_id, name).await? {
        return Ok(existing);
    }
    create(pool, user_id, name, color).await
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    color: Option<&str>,
) -> AppResult<TagRow> {
    insert(pool, user_id, name, color).await
}

pub async fn insert(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    color: Option<&str>,
) -> AppResult<TagRow> {
    let sql = "INSERT INTO tags (user_id, name, color) VALUES ($1, $2, $3)
               ON CONFLICT (user_id, normalized) DO UPDATE SET name = EXCLUDED.name
               RETURNING id, user_id, name, color, created_at";

    Ok(sqlx::query_as::<_, TagRow>(sql)
        .bind(user_id)
        .bind(name.trim())
        .bind(color)
        .fetch_one(pool)
        .await?)
}

/// Import variant that joins the caller's transaction.
pub async fn insert_in(
    connection: &mut PgConnection,
    user_id: Uuid,
    name: &str,
    color: Option<&str>,
) -> AppResult<TagRow> {
    let sql = "INSERT INTO tags (user_id, name, color) VALUES ($1, $2, $3)
               ON CONFLICT (user_id, normalized) DO UPDATE SET name = EXCLUDED.name, color = COALESCE(EXCLUDED.color, tags.color)
               RETURNING id, user_id, name, color, created_at";

    Ok(sqlx::query_as::<_, TagRow>(sql)
        .bind(user_id)
        .bind(name.trim())
        .bind(color)
        .fetch_one(&mut *connection)
        .await?)
}

pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    id: Uuid,
    name: Option<&str>,
    color: Option<&str>,
) -> AppResult<Option<TagRow>> {
    let sql = "UPDATE tags SET
                 name = COALESCE($3, name),
                 color = COALESCE($4, color)
               WHERE id = $1 AND user_id = $2
               RETURNING id, user_id, name, color, created_at";

    Ok(sqlx::query_as::<_, TagRow>(sql)
        .bind(id)
        .bind(user_id)
        .bind(name.map(str::trim))
        .bind(color)
        .fetch_optional(pool)
        .await?)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<bool> {
    let result = sqlx::query("DELETE FROM tags WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
