use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use crate::shared::{pagination::PaginationQuery, time::parse_date};

#[derive(Debug, Deserialize)]
pub struct CreateEntryRequest {
    pub date: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    pub mood_id: Option<Uuid>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEntryRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub mood_id: Option<Option<Uuid>>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// Autosave only carries what changed; status changes stay an explicit action.
#[derive(Debug, Deserialize)]
pub struct AutosaveRequest {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusRequest {
    pub status: String,
}

/// List filters. Every filter is optional and always scoped to the caller.
#[derive(Debug, Deserialize)]
pub struct EntryQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub from: Option<String>,
    pub to: Option<String>,
    pub mood_id: Option<Uuid>,
    pub tag: Option<String>,
    pub status: Option<String>,
}

impl EntryQuery {
    pub fn from_date(&self) -> Result<Option<NaiveDate>, String> {
        self.from.as_deref().map(parse_date).transpose()
    }

    pub fn to_date(&self) -> Result<Option<NaiveDate>, String> {
        self.to.as_deref().map(parse_date).transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_filters_are_parsed_or_rejected() {
        let query = EntryQuery {
            pagination: PaginationQuery {
                limit: None,
                offset: None,
            },
            from: Some("2026-10-01".into()),
            to: Some("nope".into()),
            mood_id: None,
            tag: None,
            status: None,
        };

        assert!(query.from_date().unwrap().is_some());
        assert!(query.to_date().is_err());
    }
}
