//! Cross-feature rules that belong to no single endpoint: shared validation,
//! pagination bounds, date handling, streaks and preferences.

use chrono::{Datelike, NaiveDate};
use dear_my_diary::{
    features::insights::service::streaks,
    features::settings::models::{Preferences, Theme, UpdatePreferencesRequest, Weekday},
    shared::{
        pagination::PaginationQuery,
        time::{month_range, parse_date, today},
        validation::Validator,
    },
};

fn day(month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, month, day).expect("valid test date")
}

#[test]
fn validator_reports_every_broken_field() {
    let message = Validator::new()
        .require("display_name", "", 2, 40)
        .email("email", "nope")
        .into_result()
        .expect_err("must fail")
        .to_string();

    assert!(message.contains("display_name"), "{message}");
    assert!(message.contains("email"), "{message}");
}

#[test]
fn pagination_defaults_and_bounds_are_enforced() {
    let default = PaginationQuery {
        limit: None,
        offset: None,
    }
    .normalized()
    .expect("defaults are valid");
    assert_eq!(default, (20, 0));

    assert!(PaginationQuery {
        limit: Some(500),
        offset: None
    }
    .normalized()
    .is_err());
    assert!(PaginationQuery {
        limit: Some(0),
        offset: None
    }
    .normalized()
    .is_err());
    assert!(PaginationQuery {
        limit: None,
        offset: Some(-1)
    }
    .normalized()
    .is_err());
}

#[test]
fn dates_outside_the_supported_range_are_rejected() {
    assert!(parse_date("2026-10-10").is_ok());
    assert!(parse_date("2026-02-30").is_err());
    assert!(parse_date("not-a-date").is_err());
    assert!(parse_date("1800-01-01").is_err());
}

#[test]
fn month_ranges_cover_february_and_december() {
    let (start, end) = month_range(2026, 2).expect("valid month");
    assert_eq!(start.to_string(), "2026-02-01");
    assert_eq!(end.to_string(), "2026-02-28");

    let (start, end) = month_range(2026, 12).expect("valid month");
    assert_eq!(start.to_string(), "2026-12-01");
    assert_eq!(end.to_string(), "2026-12-31");

    assert!(month_range(2026, 13).is_err());
}

#[test]
fn today_is_a_real_date() {
    let value = today();
    assert!(value.year() >= 2020);
}

#[test]
fn streaks_follow_consecutive_days() {
    assert_eq!(streaks(&[], today()), (0, 0));

    let dates = vec![day(10, 8), day(10, 9), day(10, 10)];
    assert_eq!(streaks(&dates, day(10, 10)), (3, 3));

    // A day that has not been written yet does not break the streak.
    assert_eq!(streaks(&[day(10, 8), day(10, 9)], day(10, 10)), (2, 2));

    // A stale journal breaks the current streak but keeps the record.
    assert_eq!(streaks(&[day(10, 1), day(10, 2)], day(10, 10)), (0, 2));

    let broken = vec![day(10, 1), day(10, 2), day(10, 3), day(10, 5)];
    assert_eq!(streaks(&broken, day(10, 5)), (1, 3));
}

#[test]
fn preferences_patch_only_touches_supplied_fields() {
    let mut preferences = Preferences::default();
    UpdatePreferencesRequest {
        theme: Some(Theme::Dark),
        week_starts_on: None,
        entries_per_page: None,
        show_word_count: None,
    }
    .apply(&mut preferences);

    assert_eq!(preferences.theme, Theme::Dark);
    assert_eq!(preferences.week_starts_on, Weekday::Monday);
    assert_eq!(preferences.entries_per_page, 20);
    assert!(preferences.show_word_count);
}

#[test]
fn stored_preferences_fall_back_to_defaults() {
    let parsed: Preferences = serde_json::from_str("{}").expect("defaults apply");
    assert_eq!(parsed.theme, Theme::Light);
    assert_eq!(parsed.entries_per_page, 20);
}
