use chrono::NaiveDate;
use uuid::Uuid;

/// Row shape the export needs: entry content joined with mood slug and tags.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ExportEntryRow {
    pub id: Uuid,
    pub entry_date: NaiveDate,
    pub title: String,
    pub content: String,
    pub status: String,
    pub mood_slug: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub tag_names: Option<Vec<String>>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ExportTagRow {
    pub id: Uuid,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ExportAttachmentRow {
    pub id: Uuid,
    pub entry_id: Uuid,
    pub original_name: String,
    pub content_type: String,
    pub size_bytes: i64,
}

/// One portable archive, ready to be serialized and downloaded.
#[derive(Debug, Clone)]
pub struct ExportBundle {
    pub archive: crate::shared::portable::schema::PortableArchive,
    pub file_name: String,
}
