use uuid::Uuid;

use crate::{features::settings::models::Preferences, shared::errors::AppResult};

pub async fn load(pool: &sqlx::PgPool, user_id: Uuid) -> AppResult<Preferences> {
    let raw: Option<serde_json::Value> =
        sqlx::query_scalar("SELECT preferences FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    // Unknown or missing keys fall back to defaults instead of erroring.
    Ok(raw
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default())
}

pub async fn save(pool: &sqlx::PgPool, user_id: Uuid, preferences: &Preferences) -> AppResult<()> {
    sqlx::query("UPDATE users SET preferences = $2, updated_at = now() WHERE id = $1")
        .bind(user_id)
        .bind(serde_json::to_value(preferences)?)
        .execute(pool)
        .await?;
    Ok(())
}
