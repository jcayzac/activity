use crate::types::{FocusPeriod, PointFocusEvent};

/// Merges focus periods (from knowledgeC) with focus point events (from powerlog).
/// Port of `mergeFocusStreams`.
pub fn merge_focus_streams(
    periods: &[FocusPeriod],
    point_events: &[PointFocusEvent],
) -> Vec<PointFocusEvent> {
    let from_periods: Vec<PointFocusEvent> = periods
        .iter()
        .map(|p| PointFocusEvent {
            time_ms: p.first_ms,
            bundle_id: p.bundle_id.clone(),
        })
        .collect();

    let mut sorted_periods: Vec<&FocusPeriod> = periods.iter().collect();
    sorted_periods.sort_by_key(|p| p.first_ms);

    let covered_by_period = |t: i64| -> bool {
        if sorted_periods.is_empty() {
            return false;
        }
        let mut lo: usize = 0;
        let mut hi: usize = sorted_periods.len().saturating_sub(1);
        let mut found: Option<usize> = None;
        while lo <= hi {
            let mid = lo + (hi - lo) / 2;
            if sorted_periods[mid].first_ms <= t {
                found = Some(mid);
                lo = mid + 1;
            } else {
                if mid == 0 {
                    break;
                }
                hi = mid - 1;
            }
        }
        found
            .map(|i| t < sorted_periods[i].last_ms)
            .unwrap_or(false)
    };

    let from_points: Vec<PointFocusEvent> = point_events
        .iter()
        .filter(|e| !covered_by_period(e.time_ms))
        .map(|e| PointFocusEvent {
            time_ms: e.time_ms,
            bundle_id: e.bundle_id.clone(),
        })
        .collect();

    // Merge; periods come first so they win on timestamp ties.
    let mut all: Vec<PointFocusEvent> = from_periods;
    all.extend(from_points);
    all.sort_by_key(|e| e.time_ms);

    let mut seen = std::collections::HashSet::<i64>::new();
    all.retain(|e| seen.insert(e.time_ms));
    all
}

#[cfg(test)]
mod tests {
    use super::merge_focus_streams;
    use crate::types::{FocusPeriod, PointFocusEvent};

    #[test]
    fn merge_focus_streams_suppresses_covered_points() {
        let periods = vec![FocusPeriod {
            first_ms: 1000,
            last_ms: 3000,
            bundle_id: "com.example.app".to_string(),
        }];
        let point_events = vec![
            PointFocusEvent { time_ms: 500, bundle_id: "com.other.app".to_string() },
            PointFocusEvent { time_ms: 1500, bundle_id: "com.example.app".to_string() },
            PointFocusEvent { time_ms: 4000, bundle_id: "com.other.app".to_string() },
        ];
        let result = merge_focus_streams(&periods, &point_events);
        let times: Vec<i64> = result.iter().map(|e| e.time_ms).collect();
        assert!(times.contains(&500));
        assert!(times.contains(&1000));
        assert!(times.contains(&4000));
        assert!(!times.contains(&1500), "covered point event should be suppressed");
    }

    #[test]
    fn merge_focus_streams_period_wins_on_tie() {
        let periods = vec![FocusPeriod {
            first_ms: 1000,
            last_ms: 2000,
            bundle_id: "com.period.app".to_string(),
        }];
        let point_events = vec![PointFocusEvent {
            time_ms: 1000,
            bundle_id: "com.point.app".to_string(),
        }];
        let result = merge_focus_streams(&periods, &point_events);
        let at_1000: Vec<&PointFocusEvent> = result.iter().filter(|e| e.time_ms == 1000).collect();
        assert_eq!(at_1000.len(), 1);
        assert_eq!(at_1000[0].bundle_id, "com.period.app");
    }
}
