// Pure date/time utilities, ported from lib/date-utils.ts.
//
// Key invariant: "local time" here means chrono::Local — the same timezone
// that the TypeScript runtime uses via `new Date(...)`.

use chrono::{Datelike as _, Local, NaiveDate, TimeZone as _, Timelike as _};

/// Returns `"YYYY-MM-DD"` for the given timestamp in local time.
pub fn format_local_date(ts_ms: i64) -> String {
    let dt = Local
        .timestamp_millis_opt(ts_ms)
        .single()
        .expect("timestamp out of range");
    format!("{:04}-{:02}-{:02}", dt.year(), dt.month(), dt.day())
}

/// Adds one calendar day to a `"YYYY-MM-DD"` string.
pub fn next_day(date: &str) -> String {
    let d = NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("next_day: invalid date format");
    let next = d.succ_opt().expect("next_day: date overflow");
    format!("{:04}-{:02}-{:02}", next.year(), next.month(), next.day())
}

/// Returns the ms timestamp of 06:00 local time on `date` (`"YYYY-MM-DD"`).
///
/// Mirrors `new Date(year, month-1, day, 6, 0, 0).getTime()` in TypeScript,
/// which constructs a local-time datetime.
pub fn six_am_of(date: &str) -> i64 {
    let d = NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("six_am_of: invalid date format");
    let dt = Local
        .with_ymd_and_hms(d.year(), d.month(), d.day(), 6, 0, 0)
        .single()
        .expect("six_am_of: ambiguous or invalid local time");
    dt.timestamp_millis()
}

/// Returns the "effective" calendar date for a timestamp.
///
/// Times before 06:00 local belong to the previous calendar day.
/// This mirrors the TypeScript `effectiveDay(timestampMs)`.
pub fn effective_day(ts_ms: i64) -> String {
    let dt = Local
        .timestamp_millis_opt(ts_ms)
        .single()
        .expect("effective_day: timestamp out of range");
    if dt.hour() < 6 {
        // Subtract 6 hours and take the date
        format_local_date(ts_ms - 6 * 60 * 60 * 1000)
    } else {
        format!("{:04}-{:02}-{:02}", dt.year(), dt.month(), dt.day())
    }
}

/// Returns the number of days in a month given `"YYYYMM"`.
pub fn days_in_month(yyyymm: &str) -> u32 {
    let year: i32 = yyyymm[..4].parse().expect("days_in_month: invalid year");
    let month: u32 = yyyymm[4..6].parse().expect("days_in_month: invalid month");
    // Last day of month = first day of next month minus one day
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1u32)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .expect("days_in_month: overflow")
        .pred_opt()
        .expect("days_in_month: underflow")
        .day()
}

/// Returns all `"YYYY-MM-DD"` dates in a month given `"YYYYMM"`.
pub fn build_month_dates(yyyymm: &str) -> Vec<String> {
    let year = &yyyymm[..4];
    let month = &yyyymm[4..6];
    let count = days_in_month(yyyymm);
    (1..=count)
        .map(|day| format!("{year}-{month}-{day:02}"))
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{build_month_dates, days_in_month, effective_day, next_day, six_am_of};

    #[test]
    fn six_am_round_trip() {
        // six_am_of a date, then effective_day of that timestamp should return the same date.
        let date = "2024-03-15";
        let ts = six_am_of(date);
        assert_eq!(effective_day(ts), date);
    }

    #[test]
    fn effective_day_before_6am_rolls_back() {
        // 05:59 local on 2024-03-15 should belong to 2024-03-14.
        let date = "2024-03-15";
        let six_am_ts = six_am_of(date);
        let before_6am = six_am_ts - 1; // 05:59:59.999
        assert_eq!(effective_day(before_6am), "2024-03-14");
    }

    #[test]
    fn effective_day_at_6am_stays() {
        let date = "2024-03-15";
        let ts = six_am_of(date);
        assert_eq!(effective_day(ts), date);
    }

    #[test]
    fn next_day_basic() {
        assert_eq!(next_day("2024-03-15"), "2024-03-16");
    }

    #[test]
    fn next_day_month_boundary() {
        assert_eq!(next_day("2024-01-31"), "2024-02-01");
    }

    #[test]
    fn next_day_year_boundary() {
        assert_eq!(next_day("2023-12-31"), "2024-01-01");
    }

    #[test]
    fn days_in_month_feb_leap() {
        assert_eq!(days_in_month("202402"), 29);
    }

    #[test]
    fn days_in_month_feb_non_leap() {
        assert_eq!(days_in_month("202302"), 28);
    }

    #[test]
    fn days_in_month_dec() {
        assert_eq!(days_in_month("202312"), 31);
    }

    #[test]
    fn build_month_dates_count() {
        let dates = build_month_dates("202401");
        assert_eq!(dates.len(), 31);
        assert_eq!(dates[0], "2024-01-01");
        assert_eq!(dates[30], "2024-01-31");
    }
}
