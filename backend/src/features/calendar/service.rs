use chrono::Datelike;

use crate::{
    features::calendar::{
        models::{CalendarDay, CalendarMonth},
        repository,
    },
    middleware::authentication::AuthUser,
    shared::{
        errors::{AppResult, ValidationError},
        time::{month_range, today},
    },
    state::AppState,
};

/// Day cells for one month, including days without an entry so the frontend can
/// render a complete grid.
pub async fn month(
    state: &AppState,
    caller: AuthUser,
    year: i32,
    month_number: u32,
) -> AppResult<CalendarMonth> {
    let (first, last) = month_range(year, month_number).map_err(ValidationError::new)?;
    let rows = repository::month_summary(&state.pool, caller.id, first, last).await?;

    let summary: std::collections::HashMap<_, _> = rows
        .into_iter()
        .map(|(date, words, slug)| (date, (words, slug)))
        .collect();

    let today_date = today();
    let mut days = Vec::new();
    let mut total_words = 0;
    let mut cursor = Some(first);

    while let Some(date) = cursor {
        let (word_count, mood_slug) = summary.get(&date).cloned().unwrap_or((0, None));
        total_words += word_count;

        days.push(CalendarDay {
            date,
            has_entry: summary.contains_key(&date),
            word_count,
            mood_slug,
            is_today: date == today_date,
        });

        cursor = date.succ_opt();
        if date == last {
            break;
        }
    }

    Ok(CalendarMonth {
        first_date: first,
        last_date: last,
        total_entries: days.iter().filter(|day| day.has_entry).count() as i64,
        total_words,
        days,
    })
}

/// Years that contain entries, so the calendar can offer quick navigation.
pub async fn years(state: &AppState, caller: AuthUser) -> AppResult<Vec<i32>> {
    let rows = repository::months_with_entries(&state.pool, caller.id).await?;
    let mut years: Vec<i32> = rows.iter().map(|(year, _, _)| *year).collect();
    years.sort_unstable();
    years.dedup();
    Ok(years)
}

pub fn current_year() -> i32 {
    today().year()
}
