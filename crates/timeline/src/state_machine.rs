use std::collections::HashMap;

use crate::types::{
    InitialState, Interval, IntervalLabel, ScreenEventKind, TimelineInputs, MIN_PERIOD_MS,
    SOFT_CLOSE_MS,
};

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
                AnyEvent::Bl { time, .. }
                | AnyEvent::Screen { time, .. }
                | AnyEvent::Input { time, .. }
                | AnyEvent::Focus { time, .. }
                | AnyEvent::Wifi { time, .. } => *time,
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
        all.push(AnyEvent::Bl { time: e.time_ms, active: e.active });
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
        all.push(AnyEvent::Focus { time: e.time_ms, bundle_id: e.bundle_id.clone() });
    }
    for e in wifi_events {
        all.push(AnyEvent::Wifi { time: e.time_ms, subnet: e.subnet.clone() });
    }

    all.sort_by(|a, b| {
        let t = a.time().cmp(&b.time());
        if t != std::cmp::Ordering::Equal { t } else { a.col().cmp(&b.col()) }
    });

    let events: Vec<AnyEvent> = all
        .into_iter()
        .filter(|e| e.time() >= window_start && e.time() <= window_end)
        .collect();

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

    let transition = |stack: &mut Vec<Seg>, t: i64, new_label: IntervalLabel| {
        if let Some(top) = stack.last_mut() {
            top.end = t;
        }
        stack.push(Seg { start: t, end: t, label: new_label });
    };

    let start_active = |stack: &mut Vec<Seg>, t: i64| {
        if stack.last().map(|s| s.label == IntervalLabel::Active).unwrap_or(false) {
            return;
        }
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
                stack.push(Seg { start: t, end: t, label: IntervalLabel::Active });
            }
            Some(top) if top.label == IntervalLabel::Active => {}
            Some(top) if t - top.start < MIN_PERIOD_MS => {
                top.label = IntervalLabel::Active;
                top.end = t;
            }
            _ => {
                if let Some(top) = stack.last_mut() {
                    top.end = t;
                }
                stack.push(Seg { start: t, end: t, label: IntervalLabel::Active });
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
                    if let Some(top) = stack.last_mut()
                        && top.label == IntervalLabel::Active
                        && let Some(lit) = last_input_time
                        && t - lit >= SOFT_CLOSE_MS
                    {
                        let soft_end = lit;
                        top.end = soft_end;
                        stack.push(Seg {
                            start: soft_end,
                            end: t,
                            label: IntervalLabel::Break,
                        });
                    }
                    start_active(&mut stack, t);
                    last_input_time = Some(t);
                }
            }
            AnyEvent::Input { .. } => {
                if let Some(top) = stack.last_mut()
                    && top.label == IntervalLabel::Active
                    && let Some(lit) = last_input_time
                    && t - lit >= SOFT_CLOSE_MS
                {
                    let soft_end = lit;
                    top.end = soft_end;
                    stack.push(Seg {
                        start: soft_end,
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
                            stack[top_idx].end = split_at;
                            if split_at - top_start < MIN_PERIOD_MS && stack.len() >= 2 {
                                stack[top_idx - 1].end = split_at;
                                stack.pop();
                            }
                            stack.push(Seg {
                                start: split_at,
                                end: t,
                                label: IntervalLabel::Transit,
                            });
                        } else {
                            stack[top_idx].label = IntervalLabel::Transit;
                            stack[top_idx].end = t;
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
                    stack.push(Seg { start: t, end: t, label: IntervalLabel::Break });
                }

                if let Some(new_c) = new_canon {
                    current_ip_canon = Some(new_c);
                } else if subnet.is_empty() {
                    current_ip_canon = None;
                }
            }
        }
        prev_event_time = t;
    }

    if let Some(top) = stack.last_mut()
        && top.label == IntervalLabel::Active
    {
        top.end = top.end.max(prev_event_time).max(top.start);
    }

    while stack.last().map(|s| s.label == IntervalLabel::Break).unwrap_or(false) {
        stack.pop();
    }

    let raw: Vec<(i64, i64, IntervalLabel)> = stack
        .into_iter()
        .filter(|s| s.end > s.start)
        .map(|s| (s.start, s.end, s.label))
        .collect();

    let mut merged: Vec<Interval> = Vec::new();
    for (first, last, label) in raw {
        match merged.last_mut() {
            Some(prev) if prev.label == label && first <= prev.last_ms => {
                if last > prev.last_ms {
                    prev.last_ms = last;
                }
            }
            _ => merged.push(Interval { first_ms: first, last_ms: last, label, location: None }),
        }
    }
    merged
}
