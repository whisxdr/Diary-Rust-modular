use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Insights {
    pub total_entries: i64,
    pub total_words: i64,
    pub current_streak: i64,
    pub longest_streak: i64,
    pub first_entry_date: Option<chrono::NaiveDate>,
    pub mood_distribution: Vec<MoodShare>,
    pub top_tags: Vec<TagShare>,
    pub writing_by_weekday: Vec<WeekdayShare>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MoodShare {
    pub slug: String,
    pub label: String,
    pub entries: i64,
    /// Share of entries that carry this mood, 0..1.
    pub share: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TagShare {
    pub name: String,
    pub color: Option<String>,
    pub entries: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WeekdayShare {
    /// 1 = Monday .. 7 = Sunday, matching PostgreSQL's `DOW`.
    pub weekday: i32,
    pub entries: i64,
}
