use uuid::Uuid;

use crate::{
    features::{
        attachments::{
            models::{Attachment, UploadedFile},
            repository,
        },
        entries::repository as entries_repository,
    },
    middleware::authentication::AuthUser,
    shared::{
        errors::{AppError, AppResult},
        validation::Validator,
    },
    state::AppState,
};

/// Content types a diary accepts. Anything else is stored as a download only.
const ALLOWED: [&str; 6] = [
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
    "application/pdf",
    "text/plain",
];

pub async fn upload(
    state: &AppState,
    caller: AuthUser,
    entry_id: Option<Uuid>,
    file: UploadedFile,
) -> AppResult<Attachment> {
    validate(&file)?;

    if let Some(entry_id) = entry_id {
        // The entry must belong to the caller before anything is written.
        if entries_repository::find_by_id(&state.pool, caller.id, entry_id)
            .await?
            .is_none()
        {
            return Err(AppError::NotFound("Entry not found.".into()));
        }
    }

    let stored_path = state
        .storage
        .save(caller.id, &file.file_name, &file.bytes)
        .await?;

    let row = repository::insert(
        &state.pool,
        caller.id,
        entry_id,
        &file.file_name,
        &stored_path,
        &file.content_type,
        file.bytes.len() as i64,
    )
    .await?;

    Ok(row.into())
}

pub async fn list_for_entry(
    state: &AppState,
    caller: AuthUser,
    entry_id: Uuid,
) -> AppResult<Vec<Attachment>> {
    Ok(repository::list_for_entry(&state.pool, caller.id, entry_id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

pub async fn download(
    state: &AppState,
    caller: AuthUser,
    id: Uuid,
) -> AppResult<(crate::features::attachments::models::AttachmentRow, Vec<u8>)> {
    let row = repository::find(&state.pool, caller.id, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Attachment not found.".into()))?;

    let bytes = state.storage.read(&row.stored_path).await?;
    Ok((row, bytes))
}

pub async fn delete(state: &AppState, caller: AuthUser, id: Uuid) -> AppResult<()> {
    if let Some(row) = repository::delete(&state.pool, caller.id, id).await? {
        state.storage.delete(&row.stored_path).await?;
        return Ok(());
    }
    Err(AppError::NotFound("Attachment not found.".into()))
}

fn validate(file: &UploadedFile) -> AppResult<()> {
    Validator::new()
        .require("file", &file.file_name, 1, 200)
        .check(
            ALLOWED.contains(&file.content_type.as_str()),
            "file",
            "content type is not supported",
        )
        .check(!file.bytes.is_empty(), "file", "is empty")
        .check(
            !file.file_name.contains(".."),
            "file",
            "name contains path separators",
        )
        .into_result()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(name: &str, content_type: &str) -> UploadedFile {
        UploadedFile {
            file_name: name.to_owned(),
            content_type: content_type.to_owned(),
            bytes: vec![1, 2, 3],
        }
    }

    #[test]
    fn unsupported_types_and_traversal_are_rejected() {
        assert!(validate(&file("note.txt", "text/plain")).is_ok());
        assert!(validate(&file("payload.exe", "application/octet-stream")).is_err());
        assert!(validate(&file("../escape.png", "image/png")).is_err());
    }
}
