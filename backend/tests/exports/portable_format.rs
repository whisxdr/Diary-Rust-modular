//! The portable format is a contract between export and import: what an export
//! writes must be importable again.

use chrono::Utc;
use dear_my_diary::{
    features::exports::service,
    shared::portable::schema::{
        PortableArchive, PortableEntry, PortableTag, PORTABLE_FORMAT_VERSION,
    },
};

#[test]
fn an_export_carries_the_supported_format_version() {
    let archive =
        PortableArchive::new("test".into(), Utc::now()).with_content(vec![], vec![], vec![]);
    let bytes = service::serialize(&archive).expect("archive serializes");
    let parsed: PortableArchive = serde_json::from_slice(&bytes).expect("archive parses again");

    assert_eq!(parsed.manifest.format_version, PORTABLE_FORMAT_VERSION);
    assert_eq!(service::format_version(), PORTABLE_FORMAT_VERSION);
}

#[test]
fn manifest_counts_follow_the_content() {
    let archive = PortableArchive::new("test".into(), Utc::now()).with_content(
        vec![PortableTag {
            key: "tag-1".into(),
            name: "work".into(),
            color: None,
        }],
        vec![PortableEntry {
            key: "entry-1".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 10, 10).expect("valid date"),
            title: "Day".into(),
            content: "Body".into(),
            status: "published".into(),
            mood: None,
            tags: vec!["tag-1".into()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }],
        vec![],
    );

    assert_eq!(archive.manifest.counts.entries, 1);
    assert_eq!(archive.manifest.counts.tags, 1);
    assert_eq!(archive.manifest.counts.attachments, 0);
}

#[test]
fn credentials_are_never_part_of_an_archive() {
    let archive =
        PortableArchive::new("test".into(), Utc::now()).with_content(vec![], vec![], vec![]);
    let bytes = service::serialize(&archive).expect("archive serializes");
    let text = String::from_utf8(bytes).expect("archive is utf-8");

    assert!(!text.contains("password"));
    assert!(!text.contains("email"));
}
