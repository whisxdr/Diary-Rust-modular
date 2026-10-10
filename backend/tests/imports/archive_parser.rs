//! Archive validation: the import path must reject malformed and hostile files
//! before any database work happens.

use chrono::{NaiveDate, Utc};
use dear_my_diary::{
    features::imports::parser,
    shared::portable::schema::{
        PortableArchive, PortableEntry, PortableTag, PORTABLE_FORMAT_VERSION,
    },
};

fn tag(key: &str, name: &str) -> PortableTag {
    PortableTag {
        key: key.into(),
        name: name.into(),
        color: None,
    }
}

fn entry(key: &str, date: &str, tags: Vec<&str>) -> PortableEntry {
    PortableEntry {
        key: key.into(),
        date: NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("valid test date"),
        title: "Title".into(),
        content: "Body".into(),
        status: "published".into(),
        mood: None,
        tags: tags.into_iter().map(str::to_owned).collect(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn archive(tags: Vec<PortableTag>, entries: Vec<PortableEntry>) -> PortableArchive {
    PortableArchive::new("test".into(), Utc::now()).with_content(tags, entries, vec![])
}

#[test]
fn files_that_are_not_json_are_rejected() {
    assert!(parser::parse(b"not json at all").is_err());
    assert!(parser::parse(b"[]").is_err());
    assert!(parser::parse(b"").is_err());
}

#[test]
fn foreign_and_future_formats_are_rejected() {
    let mut foreign = archive(vec![], vec![]);
    foreign.manifest.format = "some-other-app".into();
    assert!(parser::validate(&foreign).is_err());

    let mut future = archive(vec![], vec![]);
    future.manifest.format_version = PORTABLE_FORMAT_VERSION + 1;
    assert!(parser::validate(&future).is_err());
}

#[test]
fn unknown_tag_references_are_rejected() {
    let broken = archive(vec![], vec![entry("e1", "2026-10-10", vec!["ghost"])]);
    let message = parser::validate(&broken)
        .expect_err("must reject")
        .to_string();
    assert!(message.contains("unknown tag"), "{message}");
}

#[test]
fn two_entries_on_the_same_day_are_rejected() {
    let broken = archive(
        vec![],
        vec![
            entry("e1", "2026-10-10", vec![]),
            entry("e2", "2026-10-10", vec![]),
        ],
    );
    assert!(parser::validate(&broken).is_err());
}

#[test]
fn unknown_status_is_rejected() {
    let mut broken_entry = entry("e1", "2026-10-10", vec![]);
    broken_entry.status = "archived".into();
    assert!(parser::validate(&archive(vec![], vec![broken_entry])).is_err());
}

#[test]
fn oversized_bodies_are_rejected() {
    let mut big = entry("e1", "2026-10-10", vec![]);
    big.content = "x".repeat(400_001);
    assert!(parser::validate(&archive(vec![], vec![big])).is_err());
}

#[test]
fn a_well_formed_archive_is_accepted() {
    let good = archive(
        vec![tag("t1", "work")],
        vec![entry("e1", "2026-10-10", vec!["t1"])],
    );
    assert!(parser::validate(&good).is_ok());

    let bytes = serde_json::to_vec(&good).expect("archive serializes");
    assert!(parser::parse(&bytes).is_ok(), "round trip through JSON");
}

#[test]
fn unknown_fields_from_a_newer_export_are_ignored() {
    let raw = r#"{
        "manifest": {"format":"dear-my-diary/backup","format_version":1,
          "exported_at":"2026-10-10T00:00:00Z","source":"test",
          "counts":{"entries":0,"tags":0,"attachments":0},"future_field": true},
        "entries": [],
        "unknown_section": {"a": 1}
    }"#;

    let parsed: PortableArchive = serde_json::from_str(raw).expect("archive parses");
    assert!(parsed.entries.is_empty());
    assert!(parser::validate(&parsed).is_ok());
}
