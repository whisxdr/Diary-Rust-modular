//! Entry rules that do not need a database.
//!
//! These live in an integration test so they run against the public API of the
//! entries feature instead of private internals.

use dear_my_diary::features::entries::{
    models::{excerpt, word_count, EntryStatus},
    validation::{validate_content, validate_tags, validate_title, MAX_TAGS},
};
use uuid::Uuid;

#[test]
fn word_count_ignores_repeated_whitespace() {
    assert_eq!(word_count("one  two\nthree"), 3);
    assert_eq!(word_count("   "), 0);
}

#[test]
fn excerpt_is_bounded_and_readable() {
    let long = "word ".repeat(200);
    assert!(excerpt(&long).chars().count() <= 161);
    assert_eq!(excerpt("short note"), "short note");
}

#[test]
fn status_round_trips_and_rejects_unknown_values() {
    assert_eq!(EntryStatus::parse("draft"), Some(EntryStatus::Draft));
    assert_eq!(EntryStatus::Published.as_str(), "published");
    assert_eq!(EntryStatus::parse("archived"), None);
}

#[test]
fn title_limits_are_enforced() {
    assert!(validate_title("A day").is_ok());
    assert!(validate_title("").is_ok());
    assert!(validate_title(&"x".repeat(201)).is_err());
}

#[test]
fn oversized_content_is_rejected() {
    assert!(validate_content("a short memory").is_ok());
    assert!(validate_content(&"x".repeat(400_001)).is_err());
}

#[test]
fn tag_limits_are_enforced() {
    assert!(validate_tags(&["work".into(), "family".into()]).is_ok());

    let too_many = (0..=MAX_TAGS)
        .map(|i| format!("tag{i}"))
        .collect::<Vec<_>>();
    assert!(validate_tags(&too_many).is_err());
    assert!(validate_tags(&["  ".into()]).is_err());
}

#[test]
fn generated_identifiers_are_unique() {
    assert_ne!(Uuid::new_v4(), Uuid::new_v4());
}
