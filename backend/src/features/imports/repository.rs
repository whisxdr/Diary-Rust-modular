use chrono::NaiveDate;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::{
    features::imports::models::ImportJob,
    shared::{errors::AppResult, portable::schema::PortableArchive},
};

const JOB_COLUMNS: &str = "id, filename, status, summary, archive, error, created_at, completed_at";

pub async fn create_job(
    pool: &PgPool,
    user_id: Uuid,
    filename: &str,
    archive: &PortableArchive,
) -> AppResult<ImportJob> {
    let sql = format!(
        "INSERT INTO import_jobs (user_id, filename, status, archive)
         VALUES ($1, $2, 'pending', $3)
         RETURNING {JOB_COLUMNS}"
    );

    Ok(sqlx::query_as::<_, ImportJob>(&sql)
        .bind(user_id)
        .bind(filename)
        .bind(serde_json::to_value(archive)?)
        .fetch_one(pool)
        .await?)
}

pub async fn finish_job(
    pool: &PgPool,
    job_id: Uuid,
    status: &str,
    summary: serde_json::Value,
    error: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE import_jobs SET status = $2, summary = $3, error = $4, completed_at = now()
                 WHERE id = $1",
    )
    .bind(job_id)
    .bind(status)
    .bind(summary)
    .bind(error)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_job(pool: &PgPool, user_id: Uuid, job_id: Uuid) -> AppResult<Option<ImportJob>> {
    let sql = format!("SELECT {JOB_COLUMNS} FROM import_jobs WHERE id = $1 AND user_id = $2");
    Ok(sqlx::query_as::<_, ImportJob>(&sql)
        .bind(job_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?)
}

pub async fn list_jobs(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<ImportJob>> {
    let sql = format!(
        "SELECT {JOB_COLUMNS} FROM import_jobs WHERE user_id = $1 ORDER BY created_at DESC LIMIT 20"
    );
    Ok(sqlx::query_as::<_, ImportJob>(&sql)
        .bind(user_id)
        .fetch_all(pool)
        .await?)
}

/// Dates the caller already has an entry for, used to detect conflicts.
pub async fn existing_dates(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<NaiveDate>> {
    Ok(
        sqlx::query_scalar("SELECT entry_date FROM entries WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(pool)
            .await?,
    )
}

/// Resolves a tag by name inside the import transaction, creating it when new.
pub async fn tag_id_for(
    connection: &mut PgConnection,
    user_id: Uuid,
    name: &str,
) -> AppResult<Uuid> {
    let existing: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM tags WHERE user_id = $1 AND normalized = $2")
            .bind(user_id)
            .bind(name.trim().to_lowercase())
            .fetch_optional(&mut *connection)
            .await?;

    match existing {
        Some(id) => Ok(id),
        None => Ok(
            crate::features::tags::repository::insert_in(connection, user_id, name, None)
                .await?
                .id,
        ),
    }
}

/// Mood slugs available in this installation, mapped to their ids.
pub async fn mood_ids(
    pool: &PgPool,
    archive: &PortableArchive,
) -> AppResult<std::collections::HashMap<String, Uuid>> {
    let mut slugs = archive
        .entries
        .iter()
        .filter_map(|entry| entry.mood.clone())
        .filter(|slug| !slug.trim().is_empty())
        .collect::<Vec<_>>();
    slugs.sort_unstable();
    slugs.dedup();

    Ok(
        crate::features::moods::repository::ids_for_slugs(pool, &slugs)
            .await?
            .into_iter()
            .collect(),
    )
}
