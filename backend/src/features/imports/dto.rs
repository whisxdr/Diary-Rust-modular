use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// What the import would do, computed before any write happens.
#[derive(Debug, Clone, Serialize)]
pub struct ImportPreview {
    /// Identifies the staged archive; the client commits it later.
    pub job_id: Option<Uuid>,
    pub format_version: u32,
    pub exported_at: chrono::DateTime<chrono::Utc>,
    pub source: String,
    pub entries_total: usize,
    pub entries_new: usize,
    pub entries_updated: usize,
    pub tags_new: usize,
    pub date_range: Option<DateRange>,
    pub problems: Vec<Problem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DateRange {
    pub first: chrono::NaiveDate,
    pub last: chrono::NaiveDate,
}

#[derive(Debug, Clone, Serialize)]
pub struct Problem {
    pub entry_key: String,
    pub message: String,
}

/// Commit options. `merge` keeps the newer copy, `skip` leaves local entries alone.
#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConflictStrategy {
    #[default]
    Merge,
    Skip,
}

#[derive(Debug, Deserialize)]
pub struct ImportRequest {
    /// Identifier of a previously uploaded and validated archive.
    pub job_id: Uuid,
    #[serde(default)]
    pub conflict_strategy: ConflictStrategy,
}

#[derive(Debug, Serialize)]
pub struct ImportSummary {
    pub job_id: Uuid,
    pub entries_created: usize,
    pub entries_updated: usize,
    pub entries_skipped: usize,
    pub tags_created: usize,
    pub status: String,
}
