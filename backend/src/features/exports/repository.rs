use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::features::exports::models::{ExportAttachmentRow, ExportEntryRow, ExportTagRow};
use crate::shared::errors::AppResult;

pub async fn entries(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<ExportEntryRow>> {
    // Tag names are aggregated per entry so the export can be built without
    // a second round trip per entry.
    let sql = "SELECT e.id, e.entry_date, e.title, e.content, e.status,
                      m.slug AS mood_slug, e.created_at, e.updated_at,
                      ARRAY(
                        SELECT t.name FROM entry_tags et
                        JOIN tags t ON t.id = et.tag_id
                        WHERE et.entry_id = e.id
                        ORDER BY t.normalized
                      ) AS tag_names
               FROM entries e
               LEFT JOIN moods m ON m.id = e.mood_id
               WHERE e.user_id = $1
               ORDER BY e.entry_date";

    Ok(sqlx::query_as::<_, ExportEntryRow>(sql)
        .bind(user_id)
        .fetch_all(pool)
        .await?)
}

pub async fn tags(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<ExportTagRow>> {
    Ok(sqlx::query_as::<_, ExportTagRow>(
        "SELECT id, name, color FROM tags WHERE user_id = $1 ORDER BY normalized",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn attachments(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<ExportAttachmentRow>> {
    Ok(sqlx::query_as::<_, ExportAttachmentRow>(
        "SELECT id, entry_id, original_name, content_type, size_bytes
         FROM attachments WHERE user_id = $1 AND entry_id IS NOT NULL
         ORDER BY created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

/// Guards against pulling a whole life into memory in one request.
pub async fn count_entries(pool: &PgPool, user_id: Uuid) -> AppResult<i64> {
    let mut query = QueryBuilder::<Postgres>::new("SELECT count(*) FROM entries WHERE user_id = ");
    query.push_bind(user_id);
    let (total,): (i64,) = query.build_query_as().fetch_one(pool).await?;
    Ok(total)
}
