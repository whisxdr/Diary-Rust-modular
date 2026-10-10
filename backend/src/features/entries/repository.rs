use chrono::NaiveDate;
use sqlx::{PgConnection, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::{
    features::entries::models::{EntryListRow, EntryRow},
    shared::errors::AppResult,
};

const ENTRY_COLUMNS: &str =
    "id, user_id, entry_date, title, content, word_count, mood_id, status, created_at, updated_at";

/// Filter set shared by the list query and its count query.
#[derive(Debug, Clone, Default)]
pub struct EntryFilter {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub mood_id: Option<Uuid>,
    pub tag_id: Option<Uuid>,
    pub status: Option<String>,
}

pub async fn find_by_id(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<Option<EntryRow>> {
    let sql = format!("SELECT {ENTRY_COLUMNS} FROM entries WHERE id = $1 AND user_id = $2");
    Ok(sqlx::query_as::<_, EntryRow>(&sql)
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?)
}

pub async fn find_by_date(
    pool: &PgPool,
    user_id: Uuid,
    date: NaiveDate,
) -> AppResult<Option<EntryRow>> {
    let sql = format!("SELECT {ENTRY_COLUMNS} FROM entries WHERE user_id = $1 AND entry_date = $2");
    Ok(sqlx::query_as::<_, EntryRow>(&sql)
        .bind(user_id)
        .bind(date)
        .fetch_optional(pool)
        .await?)
}

/// Fields of a new entry, grouped so the write helpers stay readable.
#[derive(Debug, Clone)]
pub struct NewEntry {
    pub date: NaiveDate,
    pub title: String,
    pub content: String,
    pub word_count: i32,
    pub mood_id: Option<Uuid>,
    pub status: String,
}

/// Inserts an entry. The unique index on (user_id, entry_date) rejects a second
/// entry for the same day.
pub async fn insert(pool: &PgPool, user_id: Uuid, entry: &NewEntry) -> AppResult<EntryRow> {
    let sql = format!(
        "INSERT INTO entries (user_id, entry_date, title, content, word_count, mood_id, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING {ENTRY_COLUMNS}"
    );

    Ok(sqlx::query_as::<_, EntryRow>(&sql)
        .bind(user_id)
        .bind(entry.date)
        .bind(&entry.title)
        .bind(&entry.content)
        .bind(entry.word_count)
        .bind(entry.mood_id)
        .bind(&entry.status)
        .fetch_one(pool)
        .await?)
}

pub async fn update_content(
    pool: &PgPool,
    user_id: Uuid,
    id: Uuid,
    title: Option<&str>,
    content: Option<&str>,
    word_count: Option<i32>,
    mood_id: Option<Option<Uuid>>,
) -> AppResult<Option<EntryRow>> {
    // COALESCE keeps the current value when a field is not part of the patch.
    let sql = format!(
        "UPDATE entries SET
            title = COALESCE($3, title),
            content = COALESCE($4, content),
            word_count = COALESCE($5, word_count),
            mood_id = CASE WHEN $6::boolean THEN $7 ELSE mood_id END,
            updated_at = now()
         WHERE id = $1 AND user_id = $2
         RETURNING {ENTRY_COLUMNS}"
    );

    Ok(sqlx::query_as::<_, EntryRow>(&sql)
        .bind(id)
        .bind(user_id)
        .bind(title)
        .bind(content)
        .bind(word_count)
        .bind(mood_id.is_some())
        .bind(mood_id.flatten())
        .fetch_optional(pool)
        .await?)
}

pub async fn update_status(pool: &PgPool, user_id: Uuid, id: Uuid, status: &str) -> AppResult<()> {
    sqlx::query(
        "UPDATE entries SET status = $3, updated_at = now() WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .bind(status)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<bool> {
    let result = sqlx::query("DELETE FROM entries WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// An entry arriving from a backup file, with the timestamps it carried.
#[derive(Debug, Clone)]
pub struct ImportedEntry {
    pub date: NaiveDate,
    pub title: String,
    pub content: String,
    pub word_count: i32,
    pub mood_id: Option<Uuid>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// One entry per owner and date; used by import and by "open today's page" upsert.
pub async fn upsert_for_import(
    connection: &mut PgConnection,
    user_id: Uuid,
    entry: &ImportedEntry,
) -> AppResult<Uuid> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO entries (user_id, entry_date, title, content, word_count, mood_id, status, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         ON CONFLICT (user_id, entry_date) DO UPDATE SET
            title = EXCLUDED.title,
            content = EXCLUDED.content,
            word_count = EXCLUDED.word_count,
            mood_id = EXCLUDED.mood_id,
            status = EXCLUDED.status,
            updated_at = EXCLUDED.updated_at
         RETURNING id",
    )
    .bind(user_id)
    .bind(entry.date)
    .bind(&entry.title)
    .bind(&entry.content)
    .bind(entry.word_count)
    .bind(entry.mood_id)
    .bind(&entry.status)
    .bind(entry.created_at)
    .bind(entry.updated_at)
    .fetch_one(&mut *connection)
    .await?;

    Ok(id)
}

/// Entries the caller owns, newest first. Body text stays out of list responses.
pub async fn list(
    pool: &PgPool,
    user_id: Uuid,
    filter: &EntryFilter,
    limit: i64,
    offset: i64,
) -> AppResult<(Vec<EntryListRow>, i64)> {
    let mut query = QueryBuilder::<Postgres>::new("SELECT DISTINCT e.id, e.entry_date, e.title, e.word_count, e.mood_id, e.status, e.updated_at FROM entries e ");
    push_filters(&mut query, user_id, filter);

    query
        .push(" ORDER BY e.entry_date DESC LIMIT ")
        .push_bind(limit);
    query.push(" OFFSET ").push_bind(offset);

    let rows = query
        .build_query_as::<EntryListRow>()
        .fetch_all(pool)
        .await?;

    let mut count = QueryBuilder::<Postgres>::new("SELECT count(*) FROM entries e ");
    push_filters(&mut count, user_id, filter);
    let (total,): (i64,) = count.build_query_as().fetch_one(pool).await?;

    Ok((rows, total))
}

fn push_filters(query: &mut QueryBuilder<'_, Postgres>, user_id: Uuid, filter: &EntryFilter) {
    query.push(" WHERE e.user_id = ").push_bind(user_id);

    if let Some(from) = filter.from {
        query.push(" AND e.entry_date >= ").push_bind(from);
    }
    if let Some(to) = filter.to {
        query.push(" AND e.entry_date <= ").push_bind(to);
    }
    if let Some(mood_id) = filter.mood_id {
        query.push(" AND e.mood_id = ").push_bind(mood_id);
    }
    if let Some(status) = &filter.status {
        query.push(" AND e.status = ").push_bind(status.clone());
    }
    if let Some(tag_id) = filter.tag_id {
        query.push(
            " AND EXISTS (SELECT 1 FROM entry_tags et WHERE et.entry_id = e.id AND et.tag_id = ",
        );
        query.push_bind(tag_id);
        query.push(")");
    }
}

pub async fn count_for_user(pool: &PgPool, user_id: Uuid) -> AppResult<i64> {
    let (total,): (i64,) = sqlx::query_as("SELECT count(*) FROM entries WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    Ok(total)
}

/// Body previews for the entries feature's own list endpoint.
pub async fn contents_for(pool: &PgPool, ids: &[Uuid]) -> AppResult<Vec<(Uuid, String)>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    Ok(
        sqlx::query_as::<_, (Uuid, String)>("SELECT id, content FROM entries WHERE id = ANY($1)")
            .bind(ids)
            .fetch_all(pool)
            .await?,
    )
}
