use sqlx::PgPool;
use uuid::Uuid;

use crate::{features::search::models::SearchHit, shared::errors::AppResult};

/// Weighted full-text search over the generated `search_vector`. `websearch_to_tsquery`
/// accepts ordinary words and quoted phrases without raising a syntax error.
pub async fn search(
    pool: &PgPool,
    user_id: Uuid,
    query: &str,
    limit: i64,
    offset: i64,
) -> AppResult<(Vec<SearchHit>, i64)> {
    let sql = "SELECT e.id, e.entry_date, e.title,
                      ts_headline('simple', e.content, websearch_to_tsquery('simple', $2),
                                  'StartSel=<mark>, StopSel=</mark>, MaxWords=30, MinWords=10') AS snippet,
                      ts_rank(e.search_vector, websearch_to_tsquery('simple', $2)) AS rank
               FROM entries e
               WHERE e.user_id = $1 AND e.search_vector @@ websearch_to_tsquery('simple', $2)
               ORDER BY rank DESC, e.entry_date DESC
               LIMIT $3 OFFSET $4";

    let hits = sqlx::query_as::<_, SearchHit>(sql)
        .bind(user_id)
        .bind(query)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let (total,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM entries
         WHERE user_id = $1 AND search_vector @@ websearch_to_tsquery('simple', $2)",
    )
    .bind(user_id)
    .bind(query)
    .fetch_one(pool)
    .await?;

    Ok((hits, total))
}
