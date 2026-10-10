use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::{features::entries::models::TagSummary, shared::errors::AppResult};

/// Tag assignment for one entry. The composite foreign key guarantees both sides
/// belong to the same account.
pub async fn replace_for_entry(
    pool: &PgPool,
    user_id: Uuid,
    entry_id: Uuid,
    tag_ids: &[Uuid],
) -> AppResult<()> {
    sqlx::query("DELETE FROM entry_tags WHERE entry_id = $1")
        .bind(entry_id)
        .execute(pool)
        .await?;

    for tag_id in tag_ids {
        sqlx::query("INSERT INTO entry_tags (entry_id, tag_id, user_id) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
            .bind(entry_id)
            .bind(tag_id)
            .bind(user_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

/// Same as [`replace_for_entry`] but inside an import transaction.
pub async fn replace_for_entry_in(
    connection: &mut PgConnection,
    user_id: Uuid,
    entry_id: Uuid,
    tag_ids: &[Uuid],
) -> AppResult<()> {
    sqlx::query("DELETE FROM entry_tags WHERE entry_id = $1")
        .bind(entry_id)
        .execute(&mut *connection)
        .await?;

    for tag_id in tag_ids {
        sqlx::query("INSERT INTO entry_tags (entry_id, tag_id, user_id) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
            .bind(entry_id)
            .bind(tag_id)
            .bind(user_id)
            .execute(&mut *connection)
            .await?;
    }

    Ok(())
}

pub async fn for_entries(
    pool: &PgPool,
    user_id: Uuid,
    entry_ids: &[Uuid],
) -> AppResult<Vec<(Uuid, TagSummary)>> {
    if entry_ids.is_empty() {
        return Ok(Vec::new());
    }

    let sql = "SELECT et.entry_id, t.id, t.name, t.color
               FROM entry_tags et
               JOIN tags t ON t.id = et.tag_id
               WHERE et.user_id = $1 AND et.entry_id = ANY($2)
               ORDER BY t.normalized";

    let rows = sqlx::query_as::<_, (Uuid, Uuid, String, Option<String>)>(sql)
        .bind(user_id)
        .bind(entry_ids)
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|(entry_id, id, name, color)| (entry_id, TagSummary { id, name, color }))
        .collect())
}
