use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

pub const PORTABLE_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableArchive {
    pub manifest: Manifest,
    #[serde(default)]
    pub tags: Vec<PortableTag>,
    #[serde(default)]
    pub entries: Vec<PortableEntry>,
    /// Attachments are exported as metadata only; the bytes stay on the source device.
    #[serde(default)]
    pub attachments: Vec<PortableAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub format: String,
    pub format_version: u32,
    pub exported_at: DateTime<Utc>,
    pub source: String,
    pub counts: ManifestCounts,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManifestCounts {
    pub entries: usize,
    pub tags: usize,
    pub attachments: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableTag {
    /// Stable identifier used to reconnect tags to entries across the archive.
    pub key: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableEntry {
    pub key: String,
    pub date: NaiveDate,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub mood: Option<String>,
    /// Keys of `PortableTag`.
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableAttachment {
    pub key: String,
    pub entry_key: String,
    pub original_name: String,
    pub content_type: String,
    pub size_bytes: i64,
}

impl PortableArchive {
    pub fn new(source: String, exported_at: DateTime<Utc>) -> Self {
        Self {
            manifest: Manifest {
                format: "dear-my-diary/backup".to_owned(),
                format_version: PORTABLE_FORMAT_VERSION,
                exported_at,
                source,
                counts: ManifestCounts::default(),
            },
            tags: Vec::new(),
            entries: Vec::new(),
            attachments: Vec::new(),
        }
    }

    pub fn with_content(
        mut self,
        tags: Vec<PortableTag>,
        entries: Vec<PortableEntry>,
        attachments: Vec<PortableAttachment>,
    ) -> Self {
        self.manifest.counts = ManifestCounts {
            entries: entries.len(),
            tags: tags.len(),
            attachments: attachments.len(),
        };
        self.tags = tags;
        self.entries = entries;
        self.attachments = attachments;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_counts_follow_the_content() {
        let archive = PortableArchive::new("test".into(), Utc::now()).with_content(
            vec![PortableTag {
                key: "t1".into(),
                name: "work".into(),
                color: None,
            }],
            vec![PortableEntry {
                key: "e1".into(),
                date: NaiveDate::from_ymd_opt(2026, 10, 10).unwrap(),
                title: "Day".into(),
                content: String::new(),
                status: "published".into(),
                mood: None,
                tags: vec!["t1".into()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }],
            vec![],
        );

        assert_eq!(archive.manifest.counts.entries, 1);
        assert_eq!(archive.manifest.counts.tags, 1);
        assert_eq!(archive.manifest.format_version, PORTABLE_FORMAT_VERSION);
    }

    #[test]
    fn unknown_future_fields_are_ignored() {
        let raw = r#"{
            "manifest": {"format":"dear-my-diary/backup","format_version":1,
              "exported_at":"2026-10-10T00:00:00Z","source":"test","counts":{"entries":0,"tags":0,"attachments":0},
              "future_field": true},
            "entries": [],
            "unknown_section": {"a": 1}
        }"#;

        let archive: PortableArchive = serde_json::from_str(raw).expect("archive parses");
        assert!(archive.entries.is_empty());
    }
}
