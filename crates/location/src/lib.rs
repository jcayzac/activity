#![warn(clippy::all)]
#![forbid(unsafe_code)]

mod algorithm;
mod persistence;

pub use algorithm::order_signals;
pub use persistence::{
    cluster_at, collect_duet_periods, collect_subnet_periods, dominant_cluster, load_rto_data,
    open_rto_db, resolve_location_clusters, set_watermark, update_co_occurrences, watermark,
};
pub use timeline::{RtoBlock, RtoData};

// ---------------------------------------------------------------------------
// Crate-specific error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum LocationError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const HALF_LIFE_DAYS: f64 = 14.0;
pub const CLUSTER_THRESHOLD_SECONDS: f64 = 3600.0;
pub const MIN_OVERLAP_SECONDS: f64 = 0.0;
pub const RTO_SCHEMA_VERSION: i64 = 1;
pub const HOME_LOCATION_TYPE: &str = "ANCHOR_LOCATION_TYPE_HOME";

// ---------------------------------------------------------------------------
// Signal period type
// ---------------------------------------------------------------------------

/// A time interval during which a location signal was active. Signals are namespaced:
/// `duet:<uuid>` for Duet anchors and `subnet:<cidr>` for wifi subnets.
#[derive(Debug, Clone)]
pub struct SignalPeriod {
    pub start_ms: i64,
    pub end_ms: i64,
    pub signal: String, // "duet:<uuid>" or "subnet:<cidr>"
}
