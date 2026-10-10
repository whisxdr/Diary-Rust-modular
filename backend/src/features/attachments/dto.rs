use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct UploadQuery {
    /// Optional link to an entry; attachments can also be uploaded first and
    /// attached to an entry later.
    pub entry_id: Option<Uuid>,
}
