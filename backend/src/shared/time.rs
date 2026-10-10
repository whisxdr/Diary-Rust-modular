use chrono::{Datelike, Days, NaiveDate, Utc};

pub type DiaryDate = NaiveDate;

/// Parses `YYYY-MM-DD` and rejects dates outside a plausible diary range.
pub fn parse_date(raw: &str) -> Result<DiaryDate, String> {
    let date = NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d")
        .map_err(|_| format!("`{raw}` is not a valid date, expected YYYY-MM-DD"))?;
    if date.year() < 1900 || date.year() > 2200 {
        return Err(format!("`{raw}` is outside the supported range"));
    }
    Ok(date)
}

/// Inclusive list of dates covering one calendar month, ready for calendar grids.
pub fn month_range(year: i32, month: u32) -> Result<(DiaryDate, DiaryDate), String> {
    let first = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| format!("{year}-{month:02} is not a valid month"))?;
    let last = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .ok_or_else(|| "invalid month boundary".to_owned())?
        - Days::new(1);

    Ok((first, last))
}

pub fn today() -> DiaryDate {
    Utc::now().date_naive()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_malformed_and_out_of_range_dates() {
        assert!(parse_date("2026-02-30").is_err());
        assert!(parse_date("not-a-date").is_err());
        assert!(parse_date("1800-01-01").is_err());
        assert!(parse_date("2026-10-10").is_ok());
    }

    #[test]
    fn month_range_covers_february_and_december() {
        let (start, end) = month_range(2026, 2).unwrap();
        assert_eq!((start.day(), end.day()), (1, 28));

        let (start, end) = month_range(2026, 12).unwrap();
        assert_eq!(start.to_string(), "2026-12-01");
        assert_eq!(end.to_string(), "2026-12-31");
    }
}
