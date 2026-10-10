use serde::Serialize;
use uuid::Uuid;

/// Account profile as returned by the API.
#[derive(Debug, Clone, Serialize)]
pub struct Profile {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
