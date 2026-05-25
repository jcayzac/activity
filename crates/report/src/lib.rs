#![warn(clippy::all)]
#![forbid(unsafe_code)]

// Ported from lib/report.ts.

use std::collections::HashMap;

use timeline::prepare_intervals_for_render;

pub use timeline::Interval;
pub use timeline::IntervalLabel;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// One display interval in a day report: a slice of the timeline trimmed
/// between the first and last active interval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportInterval {
    pub first_ms: i64,
    pub last_ms: i64,
    pub label: IntervalLabel,
    pub location: Option<String>,
}

/// The complete rendered data for one calendar day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayReport {
    pub date: String,
    pub intervals: Vec<ReportInterval>,
    pub total_active_ms: i64,
    pub dominant_id: Option<String>,
    pub other_ids: Vec<String>,
}

/// A non-active interval (break or transit) that falls inside a day's active window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthBreak {
    pub first_ms: i64,
    pub last_ms: i64,
    /// Always `Break` or `Transit`.
    pub label: IntervalLabel,
}

/// One row in a month report, corresponding to one calendar day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthReportRow {
    pub date: String,
    pub first_ms: i64,
    pub last_ms: i64,
    pub total_active_ms: i64,
    /// Dominant location first, then others sorted alphabetically.
    pub locations: Vec<String>,
    pub breaks: Vec<MonthBreak>,
}

/// The complete rendered data for one calendar month.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthReport {
    pub yyyymm: String,
    pub rows: Vec<MonthReportRow>,
    pub dominant_id: Option<String>,
    pub other_ids: Vec<String>,
}

// ---------------------------------------------------------------------------
// Report builders
// ---------------------------------------------------------------------------

/// Port of `buildDayReport`.
pub fn build_day_report(
    date: &str,
    intervals: &[Interval],
    dominant_id: Option<&str>,
    other_ids: &[String],
) -> DayReport {
    let render_intervals = prepare_intervals_for_render(intervals.to_vec());

    let total_active_ms: i64 = render_intervals
        .iter()
        .filter(|iv| iv.label == IntervalLabel::Active)
        .map(|iv| iv.last_ms - iv.first_ms)
        .sum();

    // Slice from first to last active interval (inclusive), filter zero-duration.
    let non_zero: Vec<&Interval> = render_intervals
        .iter()
        .filter(|iv| iv.last_ms > iv.first_ms)
        .collect();
    let first_active = non_zero
        .iter()
        .position(|iv| iv.label == IntervalLabel::Active);
    let last_active = non_zero
        .iter()
        .rposition(|iv| iv.label == IntervalLabel::Active);

    let display_intervals: Vec<ReportInterval> = match (first_active, last_active) {
        (Some(f), Some(l)) => non_zero[f..=l]
            .iter()
            .map(|iv| ReportInterval {
                first_ms: iv.first_ms,
                last_ms: iv.last_ms,
                label: iv.label.clone(),
                location: iv.location.clone(),
            })
            .collect(),
        _ => vec![],
    };

    DayReport {
        date: date.to_string(),
        intervals: display_intervals,
        total_active_ms,
        dominant_id: dominant_id.map(|s| s.to_string()),
        other_ids: other_ids.to_vec(),
    }
}

/// Port of `buildMonthReport`.
pub fn build_month_report(
    yyyymm: &str,
    dates: &[String],
    intervals_by_date: &HashMap<String, Vec<Interval>>,
    dominant_id: Option<&str>,
    other_ids: &[String],
    today: &str,
) -> MonthReport {
    let mut rows: Vec<MonthReportRow> = Vec::new();

    for date in dates {
        if date.as_str() > today {
            break;
        }

        let raw = intervals_by_date
            .get(date)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let render_intervals = prepare_intervals_for_render(raw.to_vec());
        let active_intervals: Vec<&Interval> = render_intervals
            .iter()
            .filter(|iv| iv.label == IntervalLabel::Active)
            .collect();

        if active_intervals.is_empty() {
            continue;
        }

        let total_active_ms: i64 = active_intervals
            .iter()
            .map(|iv| iv.last_ms - iv.first_ms)
            .sum();

        // Coverage per location
        let mut coverage: HashMap<String, i64> = HashMap::new();
        for iv in &active_intervals {
            if let Some(loc) = &iv.location {
                *coverage.entry(loc.clone()).or_insert(0) += iv.last_ms - iv.first_ms;
            }
        }

        // Sort: dominant first, then alphabetical
        let mut locations: Vec<String> = coverage.into_keys().collect();
        locations.sort_by(|a, b| {
            let a_is_dominant = dominant_id.map(|d| a == d).unwrap_or(false);
            let b_is_dominant = dominant_id.map(|d| b == d).unwrap_or(false);
            if a_is_dominant {
                std::cmp::Ordering::Less
            } else if b_is_dominant {
                std::cmp::Ordering::Greater
            } else {
                a.cmp(b)
            }
        });

        let first_active = active_intervals[0];
        let last_active = active_intervals[active_intervals.len() - 1];

        // Breaks between first and last active
        let breaks: Vec<MonthBreak> = render_intervals
            .iter()
            .filter(|iv| {
                iv.label != IntervalLabel::Active
                    && iv.last_ms > iv.first_ms
                    && iv.first_ms >= first_active.first_ms
                    && iv.last_ms <= last_active.last_ms
            })
            .map(|iv| MonthBreak {
                first_ms: iv.first_ms,
                last_ms: iv.last_ms,
                label: iv.label.clone(),
            })
            .collect();

        rows.push(MonthReportRow {
            date: date.clone(),
            first_ms: first_active.first_ms,
            last_ms: last_active.last_ms,
            total_active_ms,
            locations,
            breaks,
        });
    }

    MonthReport {
        yyyymm: yyyymm.to_string(),
        rows,
        dominant_id: dominant_id.map(|s| s.to_string()),
        other_ids: other_ids.to_vec(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::build_day_report;
    use timeline::{Interval, IntervalLabel};

    fn make_interval(first_ms: i64, last_ms: i64, label: IntervalLabel) -> Interval {
        Interval {
            first_ms,
            last_ms,
            label,
            location: None,
        }
    }

    // build_day_report: no active intervals returns empty
    #[test]
    fn day_report_no_active_returns_empty() {
        let intervals = vec![
            make_interval(1_000, 2_000, IntervalLabel::Break),
            make_interval(2_000, 3_000, IntervalLabel::Transit),
        ];
        let report = build_day_report("2026-05-19", &intervals, None, &[]);
        assert!(report.intervals.is_empty());
        assert_eq!(report.total_active_ms, 0);
    }

    // build_day_report: slices from first to last active inclusive
    #[test]
    fn day_report_slices_correctly() {
        // break, active, break, active, break
        // Should return: active, break, active (the surrounding breaks are excluded)
        let intervals = vec![
            make_interval(0, 1_000, IntervalLabel::Break),
            make_interval(1_000, 2_000, IntervalLabel::Active),
            make_interval(2_000, 3_000, IntervalLabel::Break),
            make_interval(3_000, 4_000, IntervalLabel::Active),
            make_interval(4_000, 5_000, IntervalLabel::Break),
        ];
        let report = build_day_report("2026-05-19", &intervals, None, &[]);
        assert_eq!(report.intervals.len(), 3);
        assert_eq!(report.intervals[0].label, IntervalLabel::Active);
        assert_eq!(report.intervals[1].label, IntervalLabel::Break);
        assert_eq!(report.intervals[2].label, IntervalLabel::Active);
        assert_eq!(report.total_active_ms, 2_000);
    }

    // build_day_report: zero-duration intervals are filtered out
    #[test]
    fn day_report_filters_zero_duration() {
        let intervals = vec![
            make_interval(0, 0, IntervalLabel::Active), // zero-duration — filtered
            make_interval(1_000, 2_000, IntervalLabel::Active),
        ];
        let report = build_day_report("2026-05-19", &intervals, None, &[]);
        assert_eq!(report.intervals.len(), 1);
        assert_eq!(report.total_active_ms, 1_000);
    }
}
