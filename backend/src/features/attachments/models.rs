use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AttachmentRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub entry_id: Option<Uuid>,
    pub original_name: String,
    pub stored_path: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Client-facing shape. The internal storage path never leaves the server.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Attachment {
    pub id: Uuid,
    pub entry_id: Option<Uuid>,
    pub original_name: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub download_url: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<AttachmentRow> for Attachment {
    fn from(row: AttachmentRow) -> Self {
        Self {
            download_url: format!("/api/attachments/{}/download", row.id),
            id: row.id,
            entry_id: row.entry_id,
            original_name: row.original_name,
            content_type: row.content_type,
            size_bytes: row.size_bytes,
            created_at: row.created_at,
        }
    }
}

/// Bytes plus the metadata the upload handler extracted from the multipart part.
#[derive(Debug, Clone)]
pub struct UploadedFile {
    pub file_name: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
}
