use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::shared::portable::schema::PortableArchive;

/// Import job row. The validated archive is kept with the job so a commit can
/// run in a later request, and so an interrupted import can be resumed.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ImportJob {
    pub id: Uuid,
    pub filename: String,
    pub status: String,
    pub summary: serde_json::Value,
    /// Never sent to the client.
    #[serde(skip_serializing)]
    pub archive: serde_json::Value,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl ImportJob {
    pub fn staged_archive(&self) -> Result<PortableArchive, crate::shared::errors::AppError> {
        serde_json::from_value(self.archive.clone()).map_err(|e| {
            crate::shared::errors::AppError::Internal(format!("staged archive is unreadable: {e}"))
        })
    }
}
