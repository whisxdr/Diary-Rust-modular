use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::shared::errors::AppResult;

/// Day-level aggregates for one month. The entries feature owns the table; this
/// feature only reads the columns it needs for the calendar grid.
pub async fn month_summary(
    pool: &PgPool,
    user_id: Uuid,
    first: NaiveDate,
    last: NaiveDate,
) -> AppResult<Vec<(NaiveDate, i32, Option<String>)>> {
    let sql = "SELECT e.entry_date, e.word_count, m.slug
               FROM entries e
               LEFT JOIN moods m ON m.id = e.mood_id
               WHERE e.user_id = $1 AND e.entry_date BETWEEN $2 AND $3
               ORDER BY e.entry_date";

    Ok(sqlx::query_as::<_, (NaiveDate, i32, Option<String>)>(sql)
        .bind(user_id)
        .bind(first)
        .bind(last)
        .fetch_all(pool)
        .await?)
}

/// Months that contain at least one entry, used by the year view.
pub async fn months_with_entries(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<(i32, i32, i64)>> {
    let sql = "SELECT EXTRACT(YEAR FROM entry_date)::int AS year,
                      EXTRACT(MONTH FROM entry_date)::int AS month,
                      count(*) AS entries
               FROM entries
               WHERE user_id = $1
               GROUP BY 1, 2
               ORDER BY 1, 2";

    Ok(sqlx::query_as::<_, (i32, i32, i64)>(sql)
        .bind(user_id)
        .fetch_all(pool)
        .await?)
}
