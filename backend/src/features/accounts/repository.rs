use uuid::Uuid;

use crate::{features::accounts::dto::DeletionSummary, shared::errors::AppResult};

pub async fn update_display_name(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    display_name: &str,
) -> AppResult<()> {
    sqlx::query("UPDATE users SET display_name = $2, updated_at = now() WHERE id = $1")
        .bind(user_id)
        .bind(display_name)
        .execute(pool)
        .await?;
    Ok(())
}

/// Entry and attachment counts are read before the delete so the user can see
/// what the confirmation is about to remove.
pub async fn counts(pool: &sqlx::PgPool, user_id: Uuid) -> AppResult<(i64, i64)> {
    let (entries,): (i64,) = sqlx::query_as("SELECT count(*) FROM entries WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    let (attachments,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM attachments WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    Ok((entries, attachments))
}

/// Attachment paths are collected first: once the rows are gone the files cannot
/// be located for cleanup.
pub async fn delete_account(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> AppResult<(DeletionSummary, Vec<String>)> {
    let paths = attachment_paths(pool, user_id).await?;
    let summary = counts(pool, user_id).await?;

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok((
        DeletionSummary {
            entries_deleted: summary.0,
            attachments_deleted: summary.1,
        },
        paths,
    ))
}

async fn attachment_paths(pool: &sqlx::PgPool, user_id: Uuid) -> AppResult<Vec<String>> {
    Ok(
        sqlx::query_scalar("SELECT stored_path FROM attachments WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(pool)
            .await?,
    )
}
