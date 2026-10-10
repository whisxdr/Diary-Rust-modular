use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::{
    features::exports::{models::ExportBundle, repository},
    middleware::authentication::AuthUser,
    shared::{
        errors::AppResult,
        portable::schema::{
            PortableArchive, PortableAttachment, PortableEntry, PortableTag,
            PORTABLE_FORMAT_VERSION,
        },
    },
    state::AppState,
};

/// Highest number of entries one export may carry.
const MAX_ENTRIES: i64 = 100_000;

pub async fn build_archive(state: &AppState, caller: AuthUser) -> AppResult<ExportBundle> {
    let total = repository::count_entries(&state.pool, caller.id).await?;
    if total > MAX_ENTRIES {
        return Err(crate::shared::errors::AppError::PayloadTooLarge(format!(
            "export holds {total} entries, the limit is {MAX_ENTRIES}"
        )));
    }

    let entry_rows = repository::entries(&state.pool, caller.id).await?;
    let tag_rows = repository::tags(&state.pool, caller.id).await?;
    let attachment_rows = repository::attachments(&state.pool, caller.id).await?;

    // Portable keys replace database ids so an archive stays portable across
    // installations and accounts.
    let tag_keys: HashMap<Uuid, String> = tag_rows
        .iter()
        .map(|tag| (tag.id, format!("tag-{}", tag.id)))
        .collect();
    let entry_keys: HashMap<Uuid, String> = entry_rows
        .iter()
        .map(|entry| (entry.id, format!("entry-{}", entry.id)))
        .collect();
    // Tag names arrive as strings from the aggregation, so the entry tags are
    // mapped back through a name index.
    let tag_keys_by_name: HashMap<String, String> = tag_rows
        .iter()
        .map(|tag| (tag.name.trim().to_lowercase(), tag_keys[&tag.id].clone()))
        .collect();

    let tags = tag_rows
        .iter()
        .map(|tag| PortableTag {
            key: tag_keys[&tag.id].clone(),
            name: tag.name.clone(),
            color: tag.color.clone(),
        })
        .collect();

    let entries = entry_rows
        .iter()
        .map(|row| PortableEntry {
            key: entry_keys[&row.id].clone(),
            date: row.entry_date,
            title: row.title.clone(),
            content: row.content.clone(),
            status: row.status.clone(),
            mood: row.mood_slug.clone(),
            tags: row
                .tag_names
                .iter()
                .flatten()
                .filter_map(|name| tag_keys_by_name.get(&name.trim().to_lowercase()).cloned())
                .collect(),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
        .collect();

    let attachments = attachment_rows
        .iter()
        .map(|row| PortableAttachment {
            key: format!("attachment-{}", row.id),
            entry_key: entry_keys
                .get(&row.entry_id)
                .cloned()
                .unwrap_or_else(|| format!("entry-{}", row.entry_id)),
            original_name: row.original_name.clone(),
            content_type: row.content_type.clone(),
            size_bytes: row.size_bytes,
        })
        .collect();

    let exported_at = Utc::now();
    let archive = PortableArchive::new("dear-my-diary".to_owned(), exported_at).with_content(
        tags,
        entries,
        attachments,
    );

    Ok(ExportBundle {
        file_name: file_name(&exported_at),
        archive,
    })
}

/// Round-trip guarantee: the serializer used for the download is the same one
/// the importer parses.
pub fn serialize(archive: &PortableArchive) -> AppResult<Vec<u8>> {
    Ok(serde_json::to_vec_pretty(archive)?)
}

pub fn format_version() -> u32 {
    PORTABLE_FORMAT_VERSION
}

fn file_name(exported_at: &chrono::DateTime<Utc>) -> String {
    format!("dear-my-diary-{}.json", exported_at.format("%Y%m%d-%H%M%S"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_serializes_to_the_portable_format() {
        let archive =
            PortableArchive::new("test".into(), Utc::now()).with_content(vec![], vec![], vec![]);
        let bytes = serialize(&archive).expect("serializes");
        let parsed: PortableArchive = serde_json::from_slice(&bytes).expect("round trips");
        assert_eq!(parsed.manifest.format_version, PORTABLE_FORMAT_VERSION);
    }

    #[test]
    fn file_name_carries_the_timestamp() {
        let stamp = Utc::now();
        assert!(file_name(&stamp).starts_with("dear-my-diary-"));
        assert!(file_name(&stamp).ends_with(".json"));
    }
}
