use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// The mood catalog is seeded by a migration and shared by every account.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Mood {
    pub id: Uuid,
    pub slug: String,
    pub label: String,
    pub icon: String,
    pub score: i16,
    pub position: i16,
}
