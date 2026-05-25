use crate::types::{AggScreenOnBucket, RawEvent, RawEventKind};

/// Derives `RawEvent`s from hourly ScreenOn aggregate buckets, sharpened by
/// nearby keyboard events. Port of `buildAggregateEvents`.
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
            let mut inferred_start = ts + (3_600_000 - screen_on * 1000);

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
