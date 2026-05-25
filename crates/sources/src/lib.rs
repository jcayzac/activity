#![warn(clippy::all)]
#![forbid(unsafe_code)]

pub mod biome_infocus;
pub mod biome_wifi;
pub mod error;
pub mod knowledge;
pub mod powerlog;
pub mod unified_log;
pub mod wifi_log;

// Re-export public API
pub use biome_infocus::{
    InFocusEvent, all_infocus_events, import_infocus_events, infocus_coverage, open_infocus_db,
};
pub use biome_wifi::{WifiSession, collect_biome_sessions};
pub use error::SourcesError;
pub use knowledge::{
    FocusPeriod, all_focus_periods, import_knowledge_focus_periods, knowledge_coverage,
    open_knowledge_db,
};
pub use powerlog::{
    AggScreenOn, BlEvent, FocusEvent, aggregate_screen_on, all_bl_events, all_focus_events,
    open_powerlog_db,
};
pub use unified_log::{
    InputEvent, InputKind, ScreenEvent, ScreenEventKind, open_unified_log_db, read_input_events,
    read_screen_events,
};
pub use wifi_log::{IpEvent, WifiIpRow, build_location_groups, open_wifi_log_db, wifi_ip_events};
