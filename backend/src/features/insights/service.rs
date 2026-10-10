use chrono::NaiveDate;

use crate::{
    features::insights::{models::Insights, repository},
    middleware::authentication::AuthUser,
    shared::{errors::AppResult, time::today},
    state::AppState,
};

pub async fn overview(state: &AppState, caller: AuthUser) -> AppResult<Insights> {
    let (total_entries, total_words, first_entry_date) =
        repository::totals(&state.pool, caller.id).await?;
    let dates = repository::entry_dates(&state.pool, caller.id).await?;

    let (current_streak, longest_streak) = streaks(&dates, today());

    Ok(Insights {
        total_entries,
        total_words,
        current_streak,
        longest_streak,
        first_entry_date,
        mood_distribution: repository::mood_distribution(&state.pool, caller.id).await?,
        top_tags: repository::top_tags(&state.pool, caller.id, 8).await?,
        writing_by_weekday: repository::writing_by_weekday(&state.pool, caller.id).await?,
    })
}

/// Streaks are computed in memory: one query already returns every date, so a
/// window function would only add a second statement to read.
///
/// Returns (current streak, longest streak). The current streak survives a day
/// that has not been written yet, as long as yesterday was written.
pub fn streaks(dates: &[NaiveDate], today: NaiveDate) -> (i64, i64) {
    if dates.is_empty() {
        return (0, 0);
    }

    let mut longest = 1;
    let mut run = 1;
    for pair in dates.windows(2) {
        if (pair[1] - pair[0]).num_days() == 1 {
            run += 1;
        } else {
            longest = longest.max(run);
            run = 1;
        }
    }
    longest = longest.max(run);

    let last = dates[dates.len() - 1];
    let current = if last == today || last == today.pred_opt().unwrap_or(today) {
        run
    } else {
        0
    };

    (current, longest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, month, day).expect("valid test date")
    }

    #[test]
    fn empty_history_has_no_streak() {
        assert_eq!(streaks(&[], day(10, 10)), (0, 0));
    }

    #[test]
    fn consecutive_days_form_a_streak() {
        let dates = vec![day(10, 8), day(10, 9), day(10, 10)];
        assert_eq!(streaks(&dates, day(10, 10)), (3, 3));
    }

    #[test]
    fn a_missing_today_keeps_the_streak_alive() {
        let dates = vec![day(10, 8), day(10, 9)];
        assert_eq!(streaks(&dates, day(10, 10)), (2, 2));
    }

    #[test]
    fn longest_streak_survives_a_break() {
        let dates = vec![day(10, 1), day(10, 2), day(10, 3), day(10, 5)];
        assert_eq!(streaks(&dates, day(10, 5)), (1, 3));
    }

    #[test]
    fn a_stale_journal_breaks_the_current_streak() {
        let dates = vec![day(10, 1), day(10, 2)];
        assert_eq!(streaks(&dates, day(10, 10)), (0, 2));
    }
}
