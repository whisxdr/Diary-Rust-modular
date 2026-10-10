use serde::{Deserialize, Serialize};

/// User preferences. Stored as JSONB on the account row; every field has a
/// default so a partially written row still loads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub theme: Theme,
    pub week_starts_on: Weekday,
    pub entries_per_page: i64,
    pub show_word_count: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Weekday {
    #[default]
    Monday,
    Sunday,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            week_starts_on: Weekday::default(),
            entries_per_page: 20,
            show_word_count: true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdatePreferencesRequest {
    pub theme: Option<Theme>,
    pub week_starts_on: Option<Weekday>,
    pub entries_per_page: Option<i64>,
    pub show_word_count: Option<bool>,
}

impl UpdatePreferencesRequest {
    /// Patch semantics: only supplied fields change.
    pub fn apply(self, current: &mut Preferences) {
        if let Some(theme) = self.theme {
            current.theme = theme;
        }
        if let Some(weekday) = self.week_starts_on {
            current.week_starts_on = weekday;
        }
        if let Some(per_page) = self.entries_per_page {
            current.entries_per_page = per_page;
        }
        if let Some(show) = self.show_word_count {
            current.show_word_count = show;
        }
    }
}
