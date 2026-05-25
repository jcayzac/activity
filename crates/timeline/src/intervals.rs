use crate::types::{Interval, IntervalLabel, RtoBlock};

fn location_for_interval(blocks: &[RtoBlock], first_ms: i64, last_ms: i64) -> Option<&str> {
    for b in blocks {
        if b.first_ms > last_ms {
            break;
        }
        if b.last_ms > first_ms {
            return Some(&b.location);
        }
    }
    None
}

/// Assigns location to each interval using RTO evidence blocks.
/// Transit intervals are skipped. Falls back to `all_periods` if no block match.
pub fn annotate_intervals_with_location(
    intervals: Vec<Interval>,
    blocks: &[RtoBlock],
    all_periods: &[RtoBlock],
) -> Vec<Interval> {
    intervals
        .into_iter()
        .map(|iv| {
            if iv.label == IntervalLabel::Transit {
                return iv;
            }
            let location = location_for_interval(blocks, iv.first_ms, iv.last_ms)
                .or_else(|| location_for_interval(all_periods, iv.first_ms, iv.last_ms))
                .map(|s| s.to_string());
            if location.is_some() {
                Interval { location, ..iv }
            } else {
                iv
            }
        })
        .collect()
}

/// When a day has more than 2 transit intervals, the inner ones are relabeled
/// as active and merged with adjacent active intervals.
/// Port of `prepareIntervalsForRender`.
pub fn prepare_intervals_for_render(intervals: Vec<Interval>) -> Vec<Interval> {
    let transit_count = intervals
        .iter()
        .filter(|iv| iv.label == IntervalLabel::Transit)
        .count();
    if transit_count <= 2 {
        return intervals;
    }

    let transit_indices: Vec<usize> = intervals
        .iter()
        .enumerate()
        .filter(|(_, iv)| iv.label == IntervalLabel::Transit)
        .map(|(i, _)| i)
        .collect();
    let inner: std::collections::HashSet<usize> = transit_indices[1..transit_indices.len() - 1]
        .iter()
        .cloned()
        .collect();

    let relabeled: Vec<Interval> = intervals
        .into_iter()
        .enumerate()
        .map(|(i, iv)| {
            if inner.contains(&i) {
                Interval { label: IntervalLabel::Active, ..iv }
            } else {
                iv
            }
        })
        .collect();

    let mut merged: Vec<Interval> = Vec::new();
    for iv in relabeled {
        match merged.last_mut() {
            Some(prev) if prev.label == iv.label => {
                let prev_dur = prev.last_ms - prev.first_ms;
                prev.last_ms = iv.last_ms;
                if iv.label == IntervalLabel::Active
                    && let Some(iv_loc) = iv.location.as_deref()
                {
                    if prev.location.is_none() {
                        prev.location = Some(iv_loc.to_string());
                    } else if prev.location.as_deref() != Some(iv_loc) {
                        let iv_dur = iv.last_ms - iv.first_ms;
                        if iv_dur > prev_dur {
                            prev.location = Some(iv_loc.to_string());
                        }
                    }
                }
            }
            _ => merged.push(iv),
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::{prepare_intervals_for_render};
    use crate::types::{Interval, IntervalLabel};

    fn make_interval(first_ms: i64, last_ms: i64, label: IntervalLabel) -> Interval {
        Interval { first_ms, last_ms, label, location: None }
    }

    #[test]
    fn two_transits_unchanged() {
        let intervals = vec![
            make_interval(0, 100, IntervalLabel::Active),
            make_interval(100, 200, IntervalLabel::Transit),
            make_interval(200, 300, IntervalLabel::Active),
            make_interval(300, 400, IntervalLabel::Transit),
            make_interval(400, 500, IntervalLabel::Active),
        ];
        let result = prepare_intervals_for_render(intervals.clone());
        assert_eq!(result.len(), intervals.len());
        let tc = result.iter().filter(|iv| iv.label == IntervalLabel::Transit).count();
        assert_eq!(tc, 2);
    }

    #[test]
    fn one_transit_unchanged() {
        let intervals = vec![
            make_interval(0, 100, IntervalLabel::Active),
            make_interval(100, 200, IntervalLabel::Transit),
            make_interval(200, 300, IntervalLabel::Active),
        ];
        let result = prepare_intervals_for_render(intervals.clone());
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn inner_transits_relabeled() {
        let intervals = vec![
            make_interval(0, 100, IntervalLabel::Active),
            make_interval(100, 200, IntervalLabel::Transit),
            make_interval(200, 300, IntervalLabel::Active),
            make_interval(300, 400, IntervalLabel::Transit),
            make_interval(400, 500, IntervalLabel::Active),
            make_interval(500, 600, IntervalLabel::Transit),
            make_interval(600, 700, IntervalLabel::Active),
        ];
        let result = prepare_intervals_for_render(intervals);
        let tc = result.iter().filter(|iv| iv.label == IntervalLabel::Transit).count();
        assert_eq!(tc, 2, "only first and last transits should remain");
    }
}
