use chrono::NaiveDate;

use crate::shared::{
    errors::ValidationError,
    portable::schema::{PortableArchive, PORTABLE_FORMAT_VERSION},
};

/// Maximum archive size accepted, independent of the configured upload limit.
pub const MAX_ARCHIVE_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_ENTRIES: usize = 200_000;

/// Parses and validates an uploaded archive. No database work happens here, so a
/// malformed or hostile file never reaches a transaction.
pub fn parse(bytes: &[u8]) -> Result<PortableArchive, ValidationError> {
    if bytes.len() > MAX_ARCHIVE_BYTES {
        return Err(ValidationError::new("archive is too large"));
    }

    let archive: PortableArchive = serde_json::from_slice(bytes)
        .map_err(|e| ValidationError::new(format!("archive is not valid JSON: {e}")))?;

    validate(&archive)?;
    Ok(archive)
}

pub fn validate(archive: &PortableArchive) -> Result<(), ValidationError> {
    if archive.manifest.format != "dear-my-diary/backup" {
        return Err(ValidationError::new("file is not a Dear My Diary backup"));
    }
    if archive.manifest.format_version > PORTABLE_FORMAT_VERSION {
        return Err(ValidationError::new(format!(
            "archive format version {} is newer than the supported version {}",
            archive.manifest.format_version, PORTABLE_FORMAT_VERSION
        )));
    }
    if archive.entries.len() > MAX_ENTRIES {
        return Err(ValidationError::new("archive holds too many entries"));
    }

    let known_keys: Vec<&str> = archive.tags.iter().map(|tag| tag.key.as_str()).collect();
    let mut seen_dates = Vec::new();

    for entry in &archive.entries {
        validate_entry(entry, &known_keys, &mut seen_dates)?;
    }

    Ok(())
}

fn validate_entry(
    entry: &crate::shared::portable::schema::PortableEntry,
    known_tag_keys: &[&str],
    seen_dates: &mut Vec<NaiveDate>,
) -> Result<(), ValidationError> {
    if entry.key.trim().is_empty() {
        return Err(ValidationError::new("an entry has no key"));
    }
    if entry.title.chars().count() > 200 {
        return Err(ValidationError::new(format!(
            "entry `{}` has a title longer than 200 characters",
            entry.key
        )));
    }
    if entry.content.len() > 400_000 {
        return Err(ValidationError::new(format!(
            "entry `{}` has a body larger than 400 kB",
            entry.key
        )));
    }
    if entry.status != "draft" && entry.status != "published" {
        return Err(ValidationError::new(format!(
            "entry `{}` has an unknown status `{}`",
            entry.key, entry.status
        )));
    }
    if entry.tags.len() > 20 {
        return Err(ValidationError::new(format!(
            "entry `{}` has more than 20 tags",
            entry.key
        )));
    }
    for key in &entry.tags {
        if !known_tag_keys.contains(&key.as_str()) {
            return Err(ValidationError::new(format!(
                "entry `{}` references unknown tag `{key}`",
                entry.key
            )));
        }
    }
    // The destination allows one entry per day; duplicates in an archive would
    // silently overwrite each other.
    if seen_dates.contains(&entry.date) {
        return Err(ValidationError::new(format!(
            "archive holds two entries for {}",
            entry.date
        )));
    }
    seen_dates.push(entry.date);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::portable::schema::{Manifest, ManifestCounts, PortableEntry, PortableTag};
    use chrono::Utc;

    fn archive_with(entries: Vec<PortableEntry>, tags: Vec<PortableTag>) -> PortableArchive {
        PortableArchive {
            manifest: Manifest {
                format: "dear-my-diary/backup".into(),
                format_version: 1,
                exported_at: Utc::now(),
                source: "test".into(),
                counts: ManifestCounts::default(),
            },
            tags,
            entries,
            attachments: vec![],
        }
    }

    fn entry(key: &str, date: &str, tags: Vec<String>) -> PortableEntry {
        PortableEntry {
            key: key.into(),
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            title: "Title".into(),
            content: "Body".into(),
            status: "published".into(),
            mood: None,
            tags,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn rejects_files_that_are_not_json() {
        assert!(parse(b"not json at all").is_err());
        assert!(parse(b"[]").is_err());
    }

    #[test]
    fn rejects_foreign_formats_and_future_versions() {
        let mut archive = archive_with(vec![], vec![]);
        archive.manifest.format = "some-other-app".into();
        assert!(validate(&archive).is_err());

        let mut archive = archive_with(vec![], vec![]);
        archive.manifest.format_version = PORTABLE_FORMAT_VERSION + 1;
        assert!(validate(&archive).is_err());
    }

    #[test]
    fn rejects_unknown_tag_references() {
        let archive = archive_with(
            vec![entry("e1", "2026-10-10", vec!["ghost".into()])],
            vec![],
        );
        let error = validate(&archive).unwrap_err().to_string();
        assert!(error.contains("unknown tag"), "{error}");
    }

    #[test]
    fn rejects_two_entries_on_the_same_day() {
        let archive = archive_with(
            vec![
                entry("e1", "2026-10-10", vec![]),
                entry("e2", "2026-10-10", vec![]),
            ],
            vec![],
        );
        assert!(validate(&archive).is_err());
    }

    #[test]
    fn accepts_a_well_formed_archive() {
        let tags = vec![PortableTag {
            key: "t1".into(),
            name: "work".into(),
            color: None,
        }];
        let archive = archive_with(vec![entry("e1", "2026-10-10", vec!["t1".into()])], tags);
        assert!(validate(&archive).is_ok());
    }

    #[test]
    fn an_oversized_body_is_rejected() {
        let mut big = entry("e1", "2026-10-10", vec![]);
        big.content = "x".repeat(400_001);
        assert!(validate(&archive_with(vec![big], vec![])).is_err());
    }
}
