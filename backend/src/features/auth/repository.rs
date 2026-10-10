use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::features::auth::models::{Session, User};
use crate::shared::errors::AppResult;

const USER_COLUMNS: &str =
    "id, email, display_name, password_hash, preferences, created_at, updated_at";

/// Inserts a new local account. Returns `None` when the address is taken.
pub async fn insert_user(
    pool: &PgPool,
    email: &str,
    display_name: &str,
    password_hash: &str,
) -> AppResult<Option<User>> {
    let sql = format!(
        "INSERT INTO users (email, display_name, password_hash) VALUES ($1, $2, $3)
         ON CONFLICT DO NOTHING
         RETURNING {USER_COLUMNS}"
    );

    match sqlx::query_as::<_, User>(&sql)
        .bind(email)
        .bind(display_name)
        .bind(password_hash)
        .fetch_optional(pool)
        .await?
    {
        Some(user) => Ok(Some(user)),
        // Unique index on lower(email) reported a conflict.
        None => Ok(None),
    }
}

/// Uniqueness is pre-checked so the caller gets a field error instead of a 409.
pub async fn email_exists(pool: &PgPool, email: &str) -> AppResult<bool> {
    let exists: (bool,) =
        sqlx::query_as("SELECT EXISTS (SELECT 1 FROM users WHERE lower(email) = $1)")
            .bind(email)
            .fetch_one(pool)
            .await?;
    Ok(exists.0)
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> AppResult<Option<User>> {
    let sql = format!("SELECT {USER_COLUMNS} FROM users WHERE lower(email) = $1");
    Ok(sqlx::query_as::<_, User>(&sql)
        .bind(email)
        .fetch_optional(pool)
        .await?)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<User>> {
    let sql = format!("SELECT {USER_COLUMNS} FROM users WHERE id = $1");
    Ok(sqlx::query_as::<_, User>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn update_password_hash(pool: &PgPool, id: Uuid, hash: &str) -> AppResult<()> {
    sqlx::query("UPDATE users SET password_hash = $2, updated_at = now() WHERE id = $1")
        .bind(id)
        .bind(hash)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn insert_session(
    pool: &PgPool,
    user_id: Uuid,
    token_hash: &str,
    user_agent: Option<&str>,
    expires_at: DateTime<Utc>,
) -> AppResult<Session> {
    let sql =
        "INSERT INTO sessions (user_id, token_hash, user_agent, expires_at) VALUES ($1, $2, $3, $4)
               RETURNING id, user_id, token_hash, user_agent, created_at, expires_at";

    Ok(sqlx::query_as::<_, Session>(sql)
        .bind(user_id)
        .bind(token_hash)
        .bind(user_agent)
        .bind(expires_at)
        .fetch_one(pool)
        .await?)
}

/// Live session for a token hash. Expired rows are deleted as they are found.
pub async fn find_active_session(pool: &PgPool, token_hash: &str) -> AppResult<Option<Session>> {
    let sql = "DELETE FROM sessions
               WHERE expires_at <= now()
               RETURNING id";

    sqlx::query_scalar::<_, Uuid>(sql)
        .bind(token_hash)
        .fetch_optional(pool)
        .await?;

    let sql = "SELECT id, user_id, token_hash, user_agent, created_at, expires_at
               FROM sessions WHERE token_hash = $1 AND expires_at > now()";

    Ok(sqlx::query_as::<_, Session>(sql)
        .bind(token_hash)
        .fetch_optional(pool)
        .await?)
}

pub async fn delete_session(pool: &PgPool, session_id: Uuid) -> AppResult<()> {
    sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Used after a password change: every other device must sign in again.
pub async fn delete_sessions_except(pool: &PgPool, user_id: Uuid, keep: Uuid) -> AppResult<()> {
    sqlx::query("DELETE FROM sessions WHERE user_id = $1 AND id <> $2")
        .bind(user_id)
        .bind(keep)
        .execute(pool)
        .await?;
    Ok(())
}
