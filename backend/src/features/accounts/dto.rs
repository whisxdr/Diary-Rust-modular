use serde::Deserialize;

/// Profile edits. Ownership is taken from the session, never from the body.
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteAccountRequest {
    /// Typed confirmation, because account deletion cannot be undone.
    pub confirm_display_name: String,
}

/// Outcome of an account deletion so the frontend can explain what happened.
#[derive(Debug, serde::Serialize)]
pub struct DeletionSummary {
    pub entries_deleted: i64,
    pub attachments_deleted: i64,
}
