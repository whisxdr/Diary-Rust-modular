use sqlx::FromRow;
use uuid::Uuid;

/// Tag plus its usage count, as returned by the tag list endpoint.
#[derive(Debug, Clone, FromRow)]
pub struct TagWithCount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub entry_count: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct TagRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub entry_count: i64,
}
