// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Gaps between active periods shorter than this are merged.
pub const MERGE_GAP_MS: i64 = 5 * 60 * 1000;
/// Periods shorter than this are dropped as noise.
pub const MIN_PERIOD_MS: i64 = 10 * 60 * 1000;
/// A soft-opened session closes after this long without keyboard activity.
pub const SOFT_CLOSE_MS: i64 = 30 * 60 * 1000;

// ---------------------------------------------------------------------------
// Interval types
// ---------------------------------------------------------------------------

/// Classification of a time interval in the activity timeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntervalLabel {
    /// The user was actively using the computer.
    Active,
    /// The screen was off or locked, or the session was soft-closed.
    Break,
    /// The user changed networks (implies a location change).
    Transit,
}

/// A contiguous time interval with an activity label and optional location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interval {
    pub first_ms: i64,
    pub last_ms: i64,
    pub label: IntervalLabel,
    /// Cluster representative, assigned before render.
    pub location: Option<String>,
}

// ---------------------------------------------------------------------------
// Legacy aggregate types
// ---------------------------------------------------------------------------

/// Legacy raw event kinds used by the aggregate/legacy timeline path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawEventKind {
    /// Screen turned on (inferred from hourly aggregate bucket).
    HardOpen,
    /// Screen turned off (inferred from hourly aggregate bucket).
    HardClose,
    /// Keyboard or mouse activity observed.
    Kbd,
}

/// A raw hardware or input event used by the legacy aggregate timeline path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEvent {
    pub time_ms: i64,
    pub kind: RawEventKind,
}

/// A contiguous active session as produced by the legacy `build_periods` path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivePeriod {
    pub first_ms: i64,
    pub last_ms: i64,
}

// ---------------------------------------------------------------------------
// Timeline input event types
// ---------------------------------------------------------------------------

/// Backlight on/off event from the powerlog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlEvent {
    pub time_ms: i64,
    /// `true` = screen turned on; `false` = screen turned off.
    pub active: bool,
}

/// Whether the screen lock changed state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenEventKind {
    Lock,
    Unlock,
}

/// A screen-lock or screen-unlock event from the unified log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenEvent {
    pub time_ms: i64,
    pub kind: ScreenEventKind,
}

/// The kind of user input observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputKind {
    /// Physical keyboard or mouse activity.
    Kbd,
    /// An app came to the foreground (treated as implicit input).
    AppLaunch,
}

/// A keyboard, mouse, or app-launch event from the unified log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputEvent {
    pub time_ms: i64,
    pub kind: InputKind,
}

/// A frontmost-app change event, used to track active application focus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontmostEvent {
    pub time_ms: i64,
    pub bundle_id: String,
}

/// A wifi IP-acquisition event, used to detect location changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiEvent {
    pub time_ms: i64,
    pub ip: String,
    pub subnet: String,
}

/// The known device state at the start of a timeline window.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InitialState {
    /// Whether the backlight was on at the window start.
    pub bl_on: bool,
    /// Whether the screen was locked at the window start.
    pub screen_locked: bool,
    /// The canonical location (subnet cluster) active at the window start.
    pub ip_canon: Option<String>,
    /// The timestamp of the last input event before the window start.
    pub last_input_time: Option<i64>,
}

// ---------------------------------------------------------------------------
// Aggregate screen-on bucket type
// ---------------------------------------------------------------------------

/// One hourly powerlog bucket recording how many seconds the screen was on.
#[derive(Debug, Clone)]
pub struct AggScreenOnBucket {
    pub time_ms: i64,
    pub screen_on_secs: i64,
}

// ---------------------------------------------------------------------------
// Focus stream types
// ---------------------------------------------------------------------------

/// A contiguous period during which one application held focus (from knowledgeC).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusPeriod {
    pub first_ms: i64,
    pub last_ms: i64,
    pub bundle_id: String,
}

/// A point-in-time focus event: the app that became frontmost at `time_ms`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointFocusEvent {
    pub time_ms: i64,
    pub bundle_id: String,
}

// ---------------------------------------------------------------------------
// Location types (owned by timeline; location crate depends on timeline)
// ---------------------------------------------------------------------------

/// A resolved location block: a time interval with its cluster-representative location ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtoBlock {
    pub first_ms: i64,
    pub last_ms: i64,
    pub location: String,
}

/// All location data needed to annotate intervals and render the RTO column.
#[derive(Debug, Clone)]
pub struct RtoData {
    pub blocks: Vec<RtoBlock>,
    pub all_periods: Vec<RtoBlock>,
    pub dominant_id: Option<String>,
    pub other_ids: Vec<String>,
}

// ---------------------------------------------------------------------------
// HasTime trait
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
// TimelineInputs
// ---------------------------------------------------------------------------

/// All event-stream inputs required by [`crate::build_timeline`].
#[derive(Clone, Copy)]
pub struct TimelineInputs<'a> {
    pub bl_events: &'a [BlEvent],
    pub screen_events: &'a [ScreenEvent],
    pub input_events: &'a [InputEvent],
    pub frontmost_events: &'a [FrontmostEvent],
    pub wifi_events: &'a [WifiEvent],
}

