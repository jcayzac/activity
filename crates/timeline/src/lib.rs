#![warn(clippy::all)]
#![forbid(unsafe_code)]

// Ported from lib/timeline.ts and lib/intervals.ts.

pub mod date_utils;

use std::collections::HashMap;

use location::RtoBlock;

use date_utils::{next_day, six_am_of};

// ---------------------------------------------------------------------------
// Public constants
// ---------------------------------------------------------------------------

/// Gaps between active periods shorter than this are merged.
pub const MERGE_GAP_MS: i64 = 5 * 60 * 1000;
/// Periods shorter than this are dropped as noise.
pub const MIN_PERIOD_MS: i64 = 10 * 60 * 1000;
/// A soft-opened session closes after this long without keyboard activity.
pub const SOFT_CLOSE_MS: i64 = 30 * 60 * 1000;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntervalLabel {
    Active,
    Break,
    Transit,
}

#[derive(Debug, Clone)]
pub struct Interval {
    pub first_ms: i64,
    pub last_ms: i64,
    pub label: IntervalLabel,
    /// Cluster representative, assigned before render.
    pub location: Option<String>,
}

/// Legacy raw event kinds used by the aggregate/legacy timeline path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawEventKind {
    HardOpen,
    HardClose,
    Kbd,
}

#[derive(Debug, Clone)]
pub struct RawEvent {
    pub time_ms: i64,
    pub kind: RawEventKind,
}

#[derive(Debug, Clone)]
pub struct ActivePeriod {
    pub first_ms: i64,
    pub last_ms: i64,
}

// ---------------------------------------------------------------------------
// Input event types for build_timeline
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BlEvent {
    pub time_ms: i64,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenEventKind {
    Lock,
    Unlock,
}

#[derive(Debug, Clone)]
pub struct ScreenEvent {
    pub time_ms: i64,
    pub kind: ScreenEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputKind {
    Kbd,
    AppLaunch,
}

#[derive(Debug, Clone)]
pub struct InputEvent {
    pub time_ms: i64,
    pub kind: InputKind,
}

#[derive(Debug, Clone)]
pub struct FrontmostEvent {
    pub time_ms: i64,
    pub bundle_id: String,
}

#[derive(Debug, Clone)]
pub struct WifiEvent {
    pub time_ms: i64,
    pub ip: String,
    pub subnet: String,
}

#[derive(Debug, Clone, Default)]
pub struct InitialState {
    pub bl_on: bool,
    pub screen_locked: bool,
    pub ip_canon: Option<String>,
    pub last_input_time: Option<i64>,
}

// ---------------------------------------------------------------------------
// Aggregate screen-on bucket type
// ---------------------------------------------------------------------------

pub struct AggScreenOnBucket {
    pub time_ms: i64,
    pub screen_on_secs: i64,
}

// ---------------------------------------------------------------------------
// Focus stream types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FocusPeriod {
    pub first_ms: i64,
    pub last_ms: i64,
    pub bundle_id: String,
}

#[derive(Debug, Clone)]
pub struct PointFocusEvent {
    pub time_ms: i64,
    pub bundle_id: String,
}

// ---------------------------------------------------------------------------
// HasTime trait — used by last_before
// ---------------------------------------------------------------------------

pub trait HasTime {
    fn time_ms(&self) -> i64;
}

impl HasTime for RawEvent {
    fn time_ms(&self) -> i64 {
        self.time_ms
    }
}

impl HasTime for PointFocusEvent {
    fn time_ms(&self) -> i64 {
        self.time_ms
    }
}

// ---------------------------------------------------------------------------
// last_before
// ---------------------------------------------------------------------------

/// Binary search for the last element whose `.time_ms()` is strictly less
/// than `t`.  Returns `None` if no such element exists.
pub fn last_before<T: HasTime>(arr: &[T], t: i64) -> Option<&T> {
    let mut lo: usize = 0;
    let mut hi: usize = arr.len().saturating_sub(1);
    // Guard the empty-array case: if arr is empty, saturating_sub gives 0 but
    // lo > hi would never be true in the loop.
    if arr.is_empty() {
        return None;
    }
    // Standard binary search: find last index where arr[mid].time_ms() < t
    let mut result: Option<usize> = None;
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        if arr[mid].time_ms() < t {
            result = Some(mid);
            lo = mid + 1;
        } else {
            if mid == 0 {
                break;
            }
            hi = mid - 1;
        }
    }
    result.map(|i| &arr[i])
}

// ---------------------------------------------------------------------------
// build_aggregate_events
// ---------------------------------------------------------------------------

/// Derives `RawEvent`s from hourly ScreenOn aggregate buckets, sharpened by
/// nearby keyboard events.  Port of `buildAggregateEvents`.
pub fn build_aggregate_events(
    buckets: &[AggScreenOnBucket],
    kbd_events: &[RawEvent],
) -> Vec<RawEvent> {
    let mut events: Vec<RawEvent> = Vec::new();
    let mut in_session = false;

    for i in 0..buckets.len() {
        let ts = buckets[i].time_ms;
        let screen_on = buckets[i].screen_on_secs;

        if screen_on == 0 {
            if in_session {
                in_session = false;
            }
            continue;
        }

        let bucket_end = ts + 3_600_000;

        if !in_session {
            // Infer session start from screen-on duration
            let mut inferred_start = ts + (3_600_000 - screen_on * 1000);

            // Binary-search for first kbd event >= ts
            let mut lo2: usize = 0;
            let mut hi2: usize = kbd_events.len().saturating_sub(1);
            if !kbd_events.is_empty() {
                while lo2 <= hi2 {
                    let m = lo2 + (hi2 - lo2) / 2;
                    if kbd_events[m].time_ms < ts {
                        lo2 = m + 1;
                    } else {
                        if m == 0 {
                            break;
                        }
                        hi2 = m - 1;
                    }
                }
            }

            // Find the first kbd event in [ts, bucket_end)
            let first_kbd = kbd_events[lo2..]
                .iter()
                .take_while(|e| e.time_ms < bucket_end)
                .find(|e| e.kind == RawEventKind::Kbd);

            if let Some(kbd) = first_kbd
                && kbd.time_ms < inferred_start
            {
                inferred_start = kbd.time_ms;
            }

            events.push(RawEvent {
                time_ms: inferred_start,
                kind: RawEventKind::HardOpen,
            });
            in_session = true;
        }

        let next = buckets.get(i + 1);
        let is_last_in_run = screen_on < 3600
            && (next.is_none()
                || next.map(|n| n.screen_on_secs == 0).unwrap_or(false)
                || next
                    .map(|n| n.time_ms > ts + 3_600_000 + 60_000)
                    .unwrap_or(false));

        if is_last_in_run {
            let mut inferred_end = ts + screen_on * 1000;

            // Find last kbd event in [ts, bucket_end)
            let last_kbd = kbd_events
                .iter()
                .rev()
                .find(|e| e.kind == RawEventKind::Kbd && e.time_ms >= ts && e.time_ms < bucket_end);

            if let Some(kbd) = last_kbd
                && kbd.time_ms > inferred_end
            {
                inferred_end = kbd.time_ms;
            }

            events.push(RawEvent {
                time_ms: inferred_end,
                kind: RawEventKind::HardClose,
            });
            in_session = false;
        }
    }

    events
}

// ---------------------------------------------------------------------------
// build_timeline
// ---------------------------------------------------------------------------

/// All event-stream inputs for `build_timeline`.
pub struct TimelineInputs<'a> {
    pub bl_events: &'a [BlEvent],
    pub screen_events: &'a [ScreenEvent],
    pub input_events: &'a [InputEvent],
    pub frontmost_events: &'a [FrontmostEvent],
    pub wifi_events: &'a [WifiEvent],
}

/// Builds a labeled `Interval` timeline from multiple event streams.
/// Port of `buildTimeline`.
pub fn build_timeline(
    inputs: TimelineInputs<'_>,
    location_groups: &HashMap<String, String>,
    window_start: i64,
    window_end: i64,
    initial_state: InitialState,
) -> Vec<Interval> {
    let TimelineInputs {
        bl_events,
        screen_events,
        input_events,
        frontmost_events,
        wifi_events,
    } = inputs;
    // Column priorities: bl=0, screen=1, input=2, focus=3, wifi=4
    // We merge into a single Vec of tagged events, then stable-sort by (time, col).

    #[derive(Clone)]
    enum AnyEvent {
        Bl { time: i64, active: bool },
        Screen { time: i64, unlock: bool },
        Input { time: i64 },
        Focus { time: i64, bundle_id: String },
        Wifi { time: i64, subnet: String },
    }

    impl AnyEvent {
        fn time(&self) -> i64 {
            match self {
                AnyEvent::Bl { time, .. } => *time,
                AnyEvent::Screen { time, .. } => *time,
                AnyEvent::Input { time, .. } => *time,
                AnyEvent::Focus { time, .. } => *time,
                AnyEvent::Wifi { time, .. } => *time,
            }
        }
        fn col(&self) -> u8 {
            match self {
                AnyEvent::Bl { .. } => 0,
                AnyEvent::Screen { .. } => 1,
                AnyEvent::Input { .. } => 2,
                AnyEvent::Focus { .. } => 3,
                AnyEvent::Wifi { .. } => 4,
            }
        }
    }

    let mut all: Vec<AnyEvent> = Vec::with_capacity(
        bl_events.len()
            + screen_events.len()
            + input_events.len()
            + frontmost_events.len()
            + wifi_events.len(),
    );

    for e in bl_events {
        all.push(AnyEvent::Bl {
            time: e.time_ms,
            active: e.active,
        });
    }
    for e in screen_events {
        all.push(AnyEvent::Screen {
            time: e.time_ms,
            unlock: e.kind == ScreenEventKind::Unlock,
        });
    }
    for e in input_events {
        all.push(AnyEvent::Input { time: e.time_ms });
    }
    for e in frontmost_events {
        all.push(AnyEvent::Focus {
            time: e.time_ms,
            bundle_id: e.bundle_id.clone(),
        });
    }
    for e in wifi_events {
        all.push(AnyEvent::Wifi {
            time: e.time_ms,
            subnet: e.subnet.clone(),
        });
    }

    // Stable sort by (time, col) — std sort_by is stable in Rust
    all.sort_by(|a, b| {
        let t = a.time().cmp(&b.time());
        if t != std::cmp::Ordering::Equal {
            t
        } else {
            a.col().cmp(&b.col())
        }
    });

    // Filter to window
    let events: Vec<AnyEvent> = all
        .into_iter()
        .filter(|e| e.time() >= window_start && e.time() <= window_end)
        .collect();

    // State machine
    #[derive(Clone)]
    struct Seg {
        start: i64,
        end: i64,
        label: IntervalLabel,
    }

    let mut stack: Vec<Seg> = Vec::new();
    let mut bl_on = initial_state.bl_on;
    let mut screen_locked = initial_state.screen_locked;
    let mut current_ip_canon: Option<String> = initial_state.ip_canon;
    let mut prev_event_time: i64 = window_start;
    let mut last_input_time: Option<i64> = initial_state.last_input_time;

    // Closes the current top segment at t and opens a new one.
    let transition = |stack: &mut Vec<Seg>, t: i64, new_label: IntervalLabel| {
        if let Some(top) = stack.last_mut() {
            top.end = t;
        }
        stack.push(Seg {
            start: t,
            end: t,
            label: new_label,
        });
    };

    // Promotes the current state to Active, potentially collapsing short non-active segments.
    let start_active = |stack: &mut Vec<Seg>, t: i64| {
        if stack
            .last()
            .map(|s| s.label == IntervalLabel::Active)
            .unwrap_or(false)
        {
            return;
        }
        // Pop short non-active segments from the top
        while stack.len() >= 2 {
            let top = stack.last().unwrap();
            if top.label == IntervalLabel::Active {
                break;
            }
            if t - top.start >= MIN_PERIOD_MS {
                break;
            }
            stack.pop();
            if let Some(below) = stack.last_mut() {
                below.end = t;
            }
        }
        match stack.last_mut() {
            None => {
                stack.push(Seg {
                    start: t,
                    end: t,
                    label: IntervalLabel::Active,
                });
            }
            Some(top) if top.label == IntervalLabel::Active => {
                // already active — nothing to do
            }
            Some(top) if t - top.start < MIN_PERIOD_MS => {
                top.label = IntervalLabel::Active;
                top.end = t;
            }
            _ => {
                if let Some(top) = stack.last_mut() {
                    top.end = t;
                }
                stack.push(Seg {
                    start: t,
                    end: t,
                    label: IntervalLabel::Active,
                });
            }
        }
    };

    for ev in &events {
        let t = ev.time();
        match ev {
            AnyEvent::Bl { active, .. } => {
                bl_on = *active;
                if !active {
                    transition(&mut stack, t, IntervalLabel::Break);
                }
            }
            AnyEvent::Screen { unlock, .. } => {
                if !unlock {
                    screen_locked = true;
                    transition(&mut stack, t, IntervalLabel::Break);
                } else {
                    screen_locked = false;
                }
            }
            AnyEvent::Focus { bundle_id, .. } => {
                if bundle_id != "com.apple.loginwindow" {
                    if screen_locked {
                        screen_locked = false;
                    }
                    // maybe_soft_close inline
                    if let Some(top) = stack.last_mut()
                        && top.label == IntervalLabel::Active
                        && let Some(lit) = last_input_time
                        && t - lit >= SOFT_CLOSE_MS
                    {
                        let soft_end = lit;
                        top.end = soft_end;
                        let break_start = soft_end;
                        stack.push(Seg {
                            start: break_start,
                            end: t,
                            label: IntervalLabel::Break,
                        });
                    }
                    start_active(&mut stack, t);
                    last_input_time = Some(t);
                }
            }
            AnyEvent::Input { .. } => {
                // maybe_soft_close inline
                if let Some(top) = stack.last_mut()
                    && top.label == IntervalLabel::Active
                    && let Some(lit) = last_input_time
                    && t - lit >= SOFT_CLOSE_MS
                {
                    let soft_end = lit;
                    top.end = soft_end;
                    let break_start = soft_end;
                    stack.push(Seg {
                        start: break_start,
                        end: t,
                        label: IntervalLabel::Break,
                    });
                }
                if bl_on && !screen_locked {
                    start_active(&mut stack, t);
                }
                last_input_time = Some(t);
            }
            AnyEvent::Wifi { subnet, .. } => {
                // Compute canonical location for new subnet
                let new_canon: Option<String> = if subnet.is_empty() {
                    None
                } else {
                    let canon = location_groups
                        .get(subnet)
                        .cloned()
                        .unwrap_or_else(|| subnet.clone());
                    Some(canon)
                };

                if let (Some(new_c), Some(cur_c)) = (&new_canon, &current_ip_canon)
                    && new_c != cur_c
                {
                    let split_at = prev_event_time;
                    if let Some(top_idx) = stack.len().checked_sub(1) {
                        let top_start = stack[top_idx].start;
                        if split_at > top_start && split_at < t {
                            // Split the current segment at prev_event_time
                            stack[top_idx].end = split_at;
                            if split_at - top_start < MIN_PERIOD_MS && stack.len() >= 2 {
                                // The split piece is too short — merge it up into the segment below
                                stack[top_idx - 1].end = split_at;
                                stack.pop();
                            }
                            stack.push(Seg {
                                start: split_at,
                                end: t,
                                label: IntervalLabel::Transit,
                            });
                        } else {
                            // Relabel the current top as transit
                            stack[top_idx].label = IntervalLabel::Transit;
                            stack[top_idx].end = t;
                            // If the transit piece is very short, merge with the segment below
                            if stack.len() >= 2 {
                                let top_start2 = stack[top_idx].start;
                                let below_start = stack[top_idx - 1].start;
                                if top_start2 - below_start < MIN_PERIOD_MS {
                                    let merged_start = below_start;
                                    stack[top_idx].start = merged_start;
                                    stack.remove(top_idx - 1);
                                }
                            }
                        }
                    }
                    // Always push a new break segment starting at t
                    stack.push(Seg {
                        start: t,
                        end: t,
                        label: IntervalLabel::Break,
                    });
                }

                // Update canon
                if let Some(new_c) = new_canon {
                    current_ip_canon = Some(new_c);
                } else if subnet.is_empty() {
                    current_ip_canon = None;
                }
            }
        }
        prev_event_time = t;
    }

    // Close the last active segment
    if let Some(top) = stack.last_mut()
        && top.label == IntervalLabel::Active
    {
        top.end = top.end.max(prev_event_time).max(top.start);
    }

    // Remove trailing break segments
    while stack
        .last()
        .map(|s| s.label == IntervalLabel::Break)
        .unwrap_or(false)
    {
        stack.pop();
    }

    // Filter zero-duration segments and map to raw intervals
    let raw: Vec<(i64, i64, IntervalLabel)> = stack
        .into_iter()
        .filter(|s| s.end > s.start)
        .map(|s| (s.start, s.end, s.label))
        .collect();

    // Final merge: adjacent same-label with overlapping ranges
    let mut merged: Vec<Interval> = Vec::new();
    for (first, last, label) in raw {
        match merged.last_mut() {
            Some(prev) if prev.label == label && first <= prev.last_ms => {
                if last > prev.last_ms {
                    prev.last_ms = last;
                }
            }
            _ => {
                merged.push(Interval {
                    first_ms: first,
                    last_ms: last,
                    label,
                    location: None,
                });
            }
        }
    }
    merged
}

// ---------------------------------------------------------------------------
// merge_focus_streams
// ---------------------------------------------------------------------------

/// Merges focus periods (from knowledgeC) with focus point events (from powerlog).
/// Port of `mergeFocusStreams`.
pub fn merge_focus_streams(
    periods: &[FocusPeriod],
    point_events: &[PointFocusEvent],
) -> Vec<PointFocusEvent> {
    // Emit each period start as a point event
    let from_periods: Vec<PointFocusEvent> = periods
        .iter()
        .map(|p| PointFocusEvent {
            time_ms: p.first_ms,
            bundle_id: p.bundle_id.clone(),
        })
        .collect();

    // Sort periods by first_ms for binary search
    let mut sorted_periods: Vec<&FocusPeriod> = periods.iter().collect();
    sorted_periods.sort_by_key(|p| p.first_ms);

    let covered_by_period = |t: i64| -> bool {
        // Binary search for last period whose first_ms <= t
        let mut lo: usize = 0;
        let mut hi: usize = sorted_periods.len().saturating_sub(1);
        if sorted_periods.is_empty() {
            return false;
        }
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

    // Merge, sort by time (periods win on ties — fromPeriods comes first in the combined vec)
    let mut all: Vec<PointFocusEvent> = from_periods;
    all.extend(from_points);
    all.sort_by_key(|e| e.time_ms);

    // Deduplicate by time (keep first occurrence — which is the period event on ties)
    let mut seen = std::collections::HashSet::<i64>::new();
    all.retain(|e| seen.insert(e.time_ms));
    all
}

// ---------------------------------------------------------------------------
// build_periods (legacy aggregate fallback)
// ---------------------------------------------------------------------------

/// Builds active periods from raw events (legacy fallback for aggregate-only data).
/// Port of `buildPeriods`.  Takes a mutable ref because it sorts in place.
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
                    raw.push(ActivePeriod {
                        first_ms: start,
                        last_ms: ev.time_ms,
                    });
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
                    let gap_from_last_kbd = last_kbd.map(|lk| ev.time_ms - lk).unwrap_or(i64::MAX);
                    if gap_from_hard_close > 0 && gap_from_last_kbd >= SOFT_CLOSE_MS {
                        session_start = Some(ev.time_ms);
                        session_hard = false;
                    }
                } else {
                    if let Some(lk) = last_kbd
                        && ev.time_ms - lk >= SOFT_CLOSE_MS
                    {
                        raw.push(ActivePeriod {
                            first_ms: session_start.unwrap(),
                            last_ms: lk,
                        });
                        session_start = Some(ev.time_ms);
                        session_hard = false;
                    }
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
        raw.push(ActivePeriod {
            first_ms: start,
            last_ms: end,
        });
    }

    // Filter short periods
    let filtered: Vec<ActivePeriod> = raw
        .into_iter()
        .filter(|p| p.last_ms - p.first_ms >= MIN_PERIOD_MS)
        .collect();

    // Merge close periods
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

// ---------------------------------------------------------------------------
// attribute_periods_to_date
// ---------------------------------------------------------------------------

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
            // Find the first soft event in [window_start, e)
            let first = soft_events
                .iter()
                .find(|ev| ev.time_ms >= window_start && ev.time_ms < e);
            match first {
                Some(ev) => ev.time_ms,
                None => continue,
            }
        };
        if e > s {
            result.push(ActivePeriod {
                first_ms: s,
                last_ms: e,
            });
        }
    }
    result
}

// ---------------------------------------------------------------------------
// annotate_intervals_with_location
// ---------------------------------------------------------------------------

/// Returns the location active during `[first_ms, last_ms]`, or `None`.
/// `Block.last_ms` is exclusive (step-function end), so overlap requires `b.last_ms > first_ms`.
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
/// Transit intervals are skipped.  Falls back to `all_periods` if no block match.
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

// ---------------------------------------------------------------------------
// prepare_intervals_for_render
// ---------------------------------------------------------------------------

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

    // Identify inner transits (all except first and last)
    let transit_indices: Vec<usize> = intervals
        .iter()
        .enumerate()
        .filter(|(_, iv)| iv.label == IntervalLabel::Transit)
        .map(|(i, _)| i)
        .collect();
    // inner = transit_indices[1..transit_indices.len()-1]
    let inner: std::collections::HashSet<usize> = transit_indices[1..transit_indices.len() - 1]
        .iter()
        .cloned()
        .collect();

    // Relabel inner transits as active
    let relabeled: Vec<Interval> = intervals
        .into_iter()
        .enumerate()
        .map(|(i, iv)| {
            if inner.contains(&i) {
                Interval {
                    label: IntervalLabel::Active,
                    ..iv
                }
            } else {
                iv
            }
        })
        .collect();

    // Merge adjacent same-label intervals, keeping the dominant location for active merges
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

// ---------------------------------------------------------------------------
// Re-export effectiveDay for timeline consumers
// ---------------------------------------------------------------------------

pub use date_utils::effective_day as effective_day_ms;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{
        FocusPeriod, Interval, IntervalLabel, PointFocusEvent, RawEvent, RawEventKind, date_utils,
        last_before, merge_focus_streams, prepare_intervals_for_render,
    };

    // ---- last_before ----

    #[test]
    fn last_before_empty() {
        let arr: Vec<RawEvent> = vec![];
        assert!(last_before(&arr, 1000).is_none());
    }

    #[test]
    fn last_before_all_ge() {
        let arr = vec![
            RawEvent {
                time_ms: 100,
                kind: RawEventKind::Kbd,
            },
            RawEvent {
                time_ms: 200,
                kind: RawEventKind::Kbd,
            },
        ];
        assert!(last_before(&arr, 100).is_none());
        assert!(last_before(&arr, 50).is_none());
    }

    #[test]
    fn last_before_found() {
        let arr = vec![
            RawEvent {
                time_ms: 100,
                kind: RawEventKind::Kbd,
            },
            RawEvent {
                time_ms: 200,
                kind: RawEventKind::Kbd,
            },
            RawEvent {
                time_ms: 300,
                kind: RawEventKind::Kbd,
            },
        ];
        let r = last_before(&arr, 250).unwrap();
        assert_eq!(r.time_ms, 200);
    }

    #[test]
    fn last_before_boundary_exact() {
        let arr = vec![
            RawEvent {
                time_ms: 100,
                kind: RawEventKind::Kbd,
            },
            RawEvent {
                time_ms: 200,
                kind: RawEventKind::Kbd,
            },
        ];
        // t == 200 → last strictly less than 200 is 100
        let r = last_before(&arr, 200).unwrap();
        assert_eq!(r.time_ms, 100);
    }

    #[test]
    fn last_before_returns_last_element() {
        let arr = vec![
            RawEvent {
                time_ms: 100,
                kind: RawEventKind::Kbd,
            },
            RawEvent {
                time_ms: 200,
                kind: RawEventKind::Kbd,
            },
        ];
        let r = last_before(&arr, 999).unwrap();
        assert_eq!(r.time_ms, 200);
    }

    // ---- merge_focus_streams ----

    #[test]
    fn merge_focus_streams_suppresses_covered_points() {
        let periods = vec![FocusPeriod {
            first_ms: 1000,
            last_ms: 3000,
            bundle_id: "com.example.app".to_string(),
        }];
        let point_events = vec![
            PointFocusEvent {
                time_ms: 500,
                bundle_id: "com.other.app".to_string(),
            },
            PointFocusEvent {
                time_ms: 1500,
                bundle_id: "com.example.app".to_string(),
            }, // covered — should be suppressed
            PointFocusEvent {
                time_ms: 4000,
                bundle_id: "com.other.app".to_string(),
            },
        ];
        let result = merge_focus_streams(&periods, &point_events);
        // Expected: period start at 1000, points at 500 and 4000 (1500 suppressed)
        let times: Vec<i64> = result.iter().map(|e| e.time_ms).collect();
        assert!(times.contains(&500));
        assert!(times.contains(&1000));
        assert!(times.contains(&4000));
        assert!(
            !times.contains(&1500),
            "covered point event should be suppressed"
        );
    }

    #[test]
    fn merge_focus_streams_period_wins_on_tie() {
        let periods = vec![FocusPeriod {
            first_ms: 1000,
            last_ms: 2000,
            bundle_id: "com.period.app".to_string(),
        }];
        // Point event at the same time as the period start
        let point_events = vec![PointFocusEvent {
            time_ms: 1000,
            bundle_id: "com.point.app".to_string(),
        }];
        let result = merge_focus_streams(&periods, &point_events);
        // Only one event at 1000 — the period's bundle_id wins (periods come first)
        let at_1000: Vec<&PointFocusEvent> = result.iter().filter(|e| e.time_ms == 1000).collect();
        assert_eq!(at_1000.len(), 1);
        assert_eq!(at_1000[0].bundle_id, "com.period.app");
    }

    // ---- prepare_intervals_for_render ----

    fn make_interval(first_ms: i64, last_ms: i64, label: IntervalLabel) -> Interval {
        Interval {
            first_ms,
            last_ms,
            label,
            location: None,
        }
    }

    #[test]
    fn prepare_intervals_for_render_two_transits_unchanged() {
        let intervals = vec![
            make_interval(0, 100, IntervalLabel::Active),
            make_interval(100, 200, IntervalLabel::Transit),
            make_interval(200, 300, IntervalLabel::Active),
            make_interval(300, 400, IntervalLabel::Transit),
            make_interval(400, 500, IntervalLabel::Active),
        ];
        let result = prepare_intervals_for_render(intervals.clone());
        assert_eq!(result.len(), intervals.len());
        let transit_count = result
            .iter()
            .filter(|iv| iv.label == IntervalLabel::Transit)
            .count();
        assert_eq!(transit_count, 2);
    }

    #[test]
    fn prepare_intervals_for_render_one_transit_unchanged() {
        let intervals = vec![
            make_interval(0, 100, IntervalLabel::Active),
            make_interval(100, 200, IntervalLabel::Transit),
            make_interval(200, 300, IntervalLabel::Active),
        ];
        let result = prepare_intervals_for_render(intervals.clone());
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn prepare_intervals_for_render_inner_transits_relabeled() {
        let intervals = vec![
            make_interval(0, 100, IntervalLabel::Active),
            make_interval(100, 200, IntervalLabel::Transit), // first transit — kept
            make_interval(200, 300, IntervalLabel::Active),
            make_interval(300, 400, IntervalLabel::Transit), // inner — relabeled
            make_interval(400, 500, IntervalLabel::Active),
            make_interval(500, 600, IntervalLabel::Transit), // last transit — kept
            make_interval(600, 700, IntervalLabel::Active),
        ];
        let result = prepare_intervals_for_render(intervals);
        // Inner transit at [300, 400] becomes active and merges with adjacent active segments
        let transit_count = result
            .iter()
            .filter(|iv| iv.label == IntervalLabel::Transit)
            .count();
        assert_eq!(
            transit_count, 2,
            "only first and last transits should remain"
        );
    }

    // ---- six_am_of / effective_day round-trip ----

    #[test]
    fn six_am_effective_day_round_trip() {
        let date = "2024-06-01";
        let ts = date_utils::six_am_of(date);
        assert_eq!(date_utils::effective_day(ts), date);
    }
}
