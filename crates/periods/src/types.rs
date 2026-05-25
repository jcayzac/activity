use std::collections::HashMap;

use timeline::Interval;

use crate::interval_cache::IntervalCacheError;

/// The computed activity periods for a set of dates.
pub struct PeriodsResult {
    /// Labeled intervals keyed by `"YYYY-MM-DD"`.
    pub intervals_by_date: HashMap<String, Vec<Interval>>,
    /// Cluster representative of the most-seen location, if any.
    pub dominant_id: Option<String>,
    /// Other location cluster representatives, sorted.
    pub other_ids: Vec<String>,
}

/// Errors that can occur while computing activity periods.
#[derive(Debug, thiserror::Error)]
pub enum PeriodsError {
    #[error(transparent)]
    Cache(#[from] IntervalCacheError),
    #[error(transparent)]
    Source(#[from] anyhow::Error),
    #[error(transparent)]
    Sources(#[from] sources::SourcesError),
}
