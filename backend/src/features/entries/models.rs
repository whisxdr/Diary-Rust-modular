use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// Entry row as stored in PostgreSQL.
#[derive(Debug, Clone, FromRow)]
pub struct EntryRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub entry_date: NaiveDate,
    pub title: String,
    pub content: String,
    pub word_count: i32,
    pub mood_id: Option<Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Entry shape returned by the API. Owner identity stays inside the feature.
#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    pub id: Uuid,
    pub date: NaiveDate,
    pub title: String,
    pub content: String,
    pub word_count: i32,
    pub mood: Option<MoodSummary>,
    pub status: EntryStatus,
    pub tags: Vec<TagSummary>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryStatus {
    Draft,
    Published,
}

impl EntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "draft" => Some(Self::Draft),
            "published" => Some(Self::Published),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MoodSummary {
    pub id: Uuid,
    pub slug: String,
    pub label: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TagSummary {
    pub id: Uuid,
    pub name: String,
    pub color: Option<String>,
}

/// List projection: everything except the entry body, which stays out of
/// list responses to keep the payload small.
#[derive(Debug, Clone, FromRow)]
pub struct EntryListRow {
    pub id: Uuid,
    pub entry_date: NaiveDate,
    pub title: String,
    pub word_count: i32,
    pub mood_id: Option<Uuid>,
    pub status: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EntrySummary {
    pub id: Uuid,
    pub date: NaiveDate,
    pub title: String,
    pub excerpt: String,
    pub word_count: i32,
    pub status: EntryStatus,
    pub updated_at: DateTime<Utc>,
}

impl From<EntryListRow> for EntrySummary {
    fn from(row: EntryListRow) -> Self {
        Self {
            id: row.id,
            date: row.entry_date,
            title: row.title,
            excerpt: String::new(),
            word_count: row.word_count,
            status: EntryStatus::parse(&row.status).unwrap_or(EntryStatus::Published),
            updated_at: row.updated_at,
        }
    }
}

/// First 160 characters of the body, used for list previews.
pub fn excerpt(content: &str) -> String {
    const LIMIT: usize = 160;

    let mut out: String = content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .collect();
    if content.split_whitespace().count() > 28 {
        out.push('…');
    }
    out.chars().take(LIMIT).collect()
}

/// Word count drives the insights feature, so it is computed once on write.
pub fn word_count(content: &str) -> i32 {
    content.split_whitespace().count() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_count_ignores_repeated_whitespace() {
        assert_eq!(word_count("one  two\nthree"), 3);
        assert_eq!(word_count("   "), 0);
    }

    #[test]
    fn status_round_trips() {
        assert_eq!(EntryStatus::parse("draft"), Some(EntryStatus::Draft));
        assert_eq!(EntryStatus::parse("nonsense"), None);
        assert_eq!(EntryStatus::Published.as_str(), "published");
    }

    #[test]
    fn excerpt_is_bounded() {
        let long = "word ".repeat(200);
        assert!(excerpt(&long).chars().count() <= 161);
        assert_eq!(excerpt("short note"), "short note");
    }
}
