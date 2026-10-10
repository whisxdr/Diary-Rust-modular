use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::features::insights::models::{MoodShare, TagShare, WeekdayShare};
use crate::shared::errors::AppResult;

pub async fn totals(pool: &PgPool, user_id: Uuid) -> AppResult<(i64, i64, Option<NaiveDate>)> {
    let sql = "SELECT count(*), COALESCE(sum(word_count), 0), min(entry_date)
               FROM entries WHERE user_id = $1";
    Ok(sqlx::query_as::<_, (i64, i64, Option<NaiveDate>)>(sql)
        .bind(user_id)
        .fetch_one(pool)
        .await?)
}

/// Consecutive days ending today (or yesterday) that have an entry.
pub async fn entry_dates(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<NaiveDate>> {
    Ok(
        sqlx::query_scalar("SELECT entry_date FROM entries WHERE user_id = $1 ORDER BY entry_date")
            .bind(user_id)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn mood_distribution(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<MoodShare>> {
    let sql = "SELECT m.slug, m.label, count(*) AS entries
               FROM entries e
               JOIN moods m ON m.id = e.mood_id
               WHERE e.user_id = $1
               GROUP BY m.slug, m.label, m.position
               ORDER BY m.position";

    let rows = sqlx::query_as::<_, (String, String, i64)>(sql)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

    let total: i64 = rows.iter().map(|(_, _, count)| *count).sum();

    Ok(rows
        .into_iter()
        .map(|(slug, label, entries)| MoodShare {
            slug,
            label,
            entries,
            share: if total == 0 {
                0.0
            } else {
                entries as f64 / total as f64
            },
        })
        .collect())
}

pub async fn top_tags(pool: &PgPool, user_id: Uuid, limit: i64) -> AppResult<Vec<TagShare>> {
    let sql = "SELECT t.name, t.color, count(*) AS entries
               FROM entry_tags et
               JOIN tags t ON t.id = et.tag_id
               WHERE et.user_id = $1
               GROUP BY t.id, t.name, t.color
               ORDER BY entries DESC, t.normalized
               LIMIT $2";

    Ok(sqlx::query_as::<_, (String, Option<String>, i64)>(sql)
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|(name, color, entries)| TagShare {
            name,
            color,
            entries,
        })
        .collect())
}

pub async fn writing_by_weekday(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<WeekdayShare>> {
    let sql = "SELECT EXTRACT(DOW FROM entry_date)::int AS weekday, count(*) AS entries
               FROM entries WHERE user_id = $1
               GROUP BY 1 ORDER BY 1";

    Ok(sqlx::query_as::<_, (i32, i64)>(sql)
        .bind(user_id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|(weekday, entries)| WeekdayShare { weekday, entries })
        .collect())
}
