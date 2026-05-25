#![warn(clippy::all)]
#![forbid(unsafe_code)]

//! Use-case orchestration: building activity periods from macOS data sources.

pub mod interval_cache;
mod orchestrate;
mod types;

pub use orchestrate::get_periods_for_dates;
pub use types::{PeriodsError, PeriodsResult};
