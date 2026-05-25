#![warn(clippy::all)]
#![forbid(unsafe_code)]

// Ported from lib/timeline.ts and lib/intervals.ts.

pub mod date_utils;
mod aggregate;
mod focus;
mod intervals;
mod periods;
mod search;
mod state_machine;
mod types;

pub use aggregate::build_aggregate_events;
pub use date_utils::effective_day as effective_day_ms;
pub use date_utils::today;
pub use focus::merge_focus_streams;
pub use intervals::{annotate_intervals_with_location, prepare_intervals_for_render};
pub use periods::{attribute_periods_to_date, build_periods};
pub use search::last_before;
pub use state_machine::build_timeline;
pub use types::{
    ActivePeriod, AggScreenOnBucket, BlEvent, FocusPeriod, FrontmostEvent, HasTime, InitialState,
    InputEvent, InputKind, Interval, IntervalLabel, PointFocusEvent, RawEvent, RawEventKind,
    RtoBlock, RtoData, ScreenEvent, ScreenEventKind, TimelineInputs, WifiEvent, MERGE_GAP_MS,
    MIN_PERIOD_MS, SOFT_CLOSE_MS,
};
