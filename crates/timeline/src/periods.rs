use crate::date_utils::{next_day, six_am_of};
use crate::types::{ActivePeriod, RawEvent, RawEventKind, MERGE_GAP_MS, MIN_PERIOD_MS, SOFT_CLOSE_MS};

/// Builds active periods from raw events (legacy fallback for aggregate-only data).
/// Port of `buildPeriods`. Takes a mutable ref because it sorts in place.
pub fn build_periods(events: &mut [RawEvent], now_ms: i64) -> Vec<ActivePeriod> {
    events.sort_by_key(|e| e.time_ms);

    let mut raw: Vec<ActivePeriod> = Vec::new();
    let mut session_start: Option<i64> = None;
    let mut session_hard = false;
    let mut last_hard_close: Option<i64> = None;
    let mut last_kbd: Option<i64> = None;

    for ev in events.iter() {
        match ev.kind {
            RawEventKind::HardOpen => {
                if session_start.is_none() {
                    session_start = Some(ev.time_ms);
                }
                session_hard = true;
            }
            RawEventKind::HardClose => {
                if let Some(start) = session_start {
                    raw.push(ActivePeriod { first_ms: start, last_ms: ev.time_ms });
                    session_start = None;
                    session_hard = false;
                }
                last_hard_close = Some(ev.time_ms);
            }
            RawEventKind::Kbd => {
                if session_start.is_none() {
                    let gap_from_hard_close = last_hard_close
                        .map(|hc| ev.time_ms - hc)
                        .unwrap_or(i64::MAX);
                    let gap_from_last_kbd =
                        last_kbd.map(|lk| ev.time_ms - lk).unwrap_or(i64::MAX);
                    if gap_from_hard_close > 0 && gap_from_last_kbd >= SOFT_CLOSE_MS {
                        session_start = Some(ev.time_ms);
                        session_hard = false;
                    }
                } else if let Some(lk) = last_kbd
                    && ev.time_ms - lk >= SOFT_CLOSE_MS
                {
                    raw.push(ActivePeriod {
                        first_ms: session_start.unwrap(),
                        last_ms: lk,
                    });
                    session_start = Some(ev.time_ms);
                    session_hard = false;
                }
                last_kbd = Some(ev.time_ms);
            }
        }
    }

    if let Some(start) = session_start {
        let end = if session_hard {
            now_ms
        } else if let Some(lk) = last_kbd {
            if lk > start { lk } else { now_ms }
        } else {
            now_ms
        };
        raw.push(ActivePeriod { first_ms: start, last_ms: end });
    }

    let filtered: Vec<ActivePeriod> = raw
        .into_iter()
        .filter(|p| p.last_ms - p.first_ms >= MIN_PERIOD_MS)
        .collect();

    let mut merged: Vec<ActivePeriod> = Vec::new();
    for p in filtered {
        match merged.last_mut() {
            Some(prev) if p.first_ms - prev.last_ms <= MERGE_GAP_MS => {
                if p.last_ms > prev.last_ms {
                    prev.last_ms = p.last_ms;
                }
            }
            _ => merged.push(p),
        }
    }
    merged
}

/// Clips active periods to a `date`'s 06:00–06:00 window.
/// Port of `attributePeriodsToDate`.
pub fn attribute_periods_to_date(
    periods: &[ActivePeriod],
    date: &str,
    soft_events: &[RawEvent],
) -> Vec<ActivePeriod> {
    let window_start = six_am_of(date);
    let window_end = six_am_of(&next_day(date));
    let mut result: Vec<ActivePeriod> = Vec::new();

    for p in periods {
        let e = p.last_ms.min(window_end);
        if e <= window_start {
            continue;
        }
        let s = if p.first_ms >= window_start {
            p.first_ms
        } else {
            let first = soft_events
                .iter()
                .find(|ev| ev.time_ms >= window_start && ev.time_ms < e);
            match first {
                Some(ev) => ev.time_ms,
                None => continue,
            }
        };
        if e > s {
            result.push(ActivePeriod { first_ms: s, last_ms: e });
        }
    }
    result
}
