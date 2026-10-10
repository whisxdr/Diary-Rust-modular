use sqlx::PgPool;
use uuid::Uuid;

use crate::{features::attachments::models::AttachmentRow, shared::errors::AppResult};

const COLUMNS: &str =
    "id, user_id, entry_id, original_name, stored_path, content_type, size_bytes, created_at";

pub async fn insert(
    pool: &PgPool,
    user_id: Uuid,
    entry_id: Option<Uuid>,
    original_name: &str,
    stored_path: &str,
    content_type: &str,
    size_bytes: i64,
) -> AppResult<AttachmentRow> {
    let sql = format!(
        "INSERT INTO attachments (user_id, entry_id, original_name, stored_path, content_type, size_bytes)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING {COLUMNS}"
    );

    Ok(sqlx::query_as::<_, AttachmentRow>(&sql)
        .bind(user_id)
        .bind(entry_id)
        .bind(original_name)
        .bind(stored_path)
        .bind(content_type)
        .bind(size_bytes)
        .fetch_one(pool)
        .await?)
}

/// Ownership is part of every lookup, so a guessed id cannot read another
/// account's file.
pub async fn find(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<Option<AttachmentRow>> {
    let sql = format!("SELECT {COLUMNS} FROM attachments WHERE id = $1 AND user_id = $2");
    Ok(sqlx::query_as::<_, AttachmentRow>(&sql)
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?)
}

pub async fn list_for_entry(
    pool: &PgPool,
    user_id: Uuid,
    entry_id: Uuid,
) -> AppResult<Vec<AttachmentRow>> {
    let sql = format!("SELECT {COLUMNS} FROM attachments WHERE user_id = $1 AND entry_id = $2 ORDER BY created_at");
    Ok(sqlx::query_as::<_, AttachmentRow>(&sql)
        .bind(user_id)
        .bind(entry_id)
        .fetch_all(pool)
        .await?)
}

pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<Option<AttachmentRow>> {
    let sql = format!("DELETE FROM attachments WHERE id = $1 AND user_id = $2 RETURNING {COLUMNS}");
    Ok(sqlx::query_as::<_, AttachmentRow>(&sql)
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?)
}
