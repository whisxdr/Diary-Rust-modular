use chrono::NaiveDate;
use serde::Serialize;

/// One day of the calendar grid.
#[derive(Debug, Clone, Serialize)]
pub struct CalendarDay {
    pub date: NaiveDate,
    pub has_entry: bool,
    pub word_count: i32,
    pub mood_slug: Option<String>,
    pub is_today: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CalendarMonth {
    pub first_date: NaiveDate,
    pub last_date: NaiveDate,
    pub days: Vec<CalendarDay>,
    pub total_entries: i64,
    pub total_words: i32,
}
