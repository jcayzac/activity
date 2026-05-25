//! Reads macOS unified log events relevant to activity tracking.
//!
//! Each event class is tracked independently, so new classes can be added later.
//! Coverage is tracked per-class; missing gaps are filled by spawning `log show`.

use anyhow::Context as _;
use camino::Utf8Path;
use chrono::TimeZone as _;
use rusqlite::Connection;
use tokio::process::Command;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

pub use timeline::InputEvent;
pub use timeline::InputKind as InputEventKind;
pub use timeline::ScreenEvent;
pub use timeline::ScreenEventKind;

// ---------------------------------------------------------------------------
// Event class definitions
// ---------------------------------------------------------------------------

struct EventClass {
    name: &'static str,
    predicate: &'static str,
}

const EVENT_CLASSES: &[EventClass] = &[
    EventClass {
        name: "keyboard",
        predicate: r#"subsystem == "com.apple.SkyLight" AND category == "KeyboardEvent""#,
    },
    EventClass {
        name: "screen_lock",
        predicate: r#"process == "loginwindow" AND eventMessage CONTAINS "sendDistributedNotification: com.apple.screenIs""#,
    },
    EventClass {
        name: "app_launch",
        predicate: r#"subsystem == "com.apple.launchservices" AND category == "open" AND eventMessage BEGINSWITH "LAUNCH:""#,
    },
];

fn kind_of(class_name: &str, line: &str) -> Option<&'static str> {
    match class_name {
        "keyboard" => Some("kbd"),
        "screen_lock" => {
            if line.contains("screenIsLocked") {
                Some("screen_lock")
            } else if line.contains("screenIsUnlocked") {
                Some("screen_unlock")
            } else {
                None
            }
        }
        "app_launch" => Some("app_launch"),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Cache schema
// ---------------------------------------------------------------------------

fn ensure_schema(db: &Connection) -> anyhow::Result<()> {
    db.execute_batch(
        "PRAGMA journal_mode=WAL;
         CREATE TABLE IF NOT EXISTS events (
             timestamp INTEGER NOT NULL,
             kind      TEXT    NOT NULL,
             PRIMARY KEY (timestamp, kind)
         );
         CREATE TABLE IF NOT EXISTS coverage (
             class    TEXT    NOT NULL PRIMARY KEY,
             earliest INTEGER NOT NULL,
             latest   INTEGER NOT NULL
         );",
    )
    .context("failed to initialise unified-log cache schema")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Coverage helpers
// ---------------------------------------------------------------------------

fn get_coverage(db: &Connection, class: &str) -> anyhow::Result<Option<(i64, i64)>> {
    let mut stmt = db.prepare_cached("SELECT earliest, latest FROM coverage WHERE class = ?1")?;
    let mut rows = stmt.query([class])?;
    if let Some(row) = rows.next()? {
        Ok(Some((row.get(0)?, row.get(1)?)))
    } else {
        Ok(None)
    }
}

fn update_coverage(db: &Connection, class: &str, earliest: i64, latest: i64) -> anyhow::Result<()> {
    db.execute(
        "INSERT INTO coverage (class, earliest, latest) VALUES (?1, ?2, ?3)
         ON CONFLICT (class) DO UPDATE SET
             earliest = MIN(earliest, excluded.earliest),
             latest   = MAX(latest,   excluded.latest)",
        rusqlite::params![class, earliest, latest],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Line parsing
// ---------------------------------------------------------------------------

/// Parses a `log show --style compact` line.
/// Returns `(timestamp_ms_local, kind_str)` or `None` if not a matching line.
fn parse_line(class_name: &str, line: &str) -> Option<(i64, &'static str)> {
    // Compact format starts with: "YYYY-MM-DD HH:MM:SS..."
    if line.len() < 19 {
        return None;
    }
    let y: i32 = line[0..4].parse().ok()?;
    let mo: u32 = line[5..7].parse().ok()?;
    let d: u32 = line[8..10].parse().ok()?;
    let h: u32 = line[11..13].parse().ok()?;
    let mi: u32 = line[14..16].parse().ok()?;
    let s: u32 = line[17..19].parse().ok()?;

    // Validate separators
    if &line[4..5] != "-"
        || &line[7..8] != "-"
        || &line[10..11] != " "
        || &line[13..14] != ":"
        || &line[16..17] != ":"
    {
        return None;
    }

    let dt = chrono::Local
        .with_ymd_and_hms(y, mo, d, h, mi, s)
        .single()?;
    let ts_ms = dt.timestamp_millis();

    let kind = kind_of(class_name, line)?;
    Some((ts_ms, kind))
}

// ---------------------------------------------------------------------------
// Fetch from `log show`
// ---------------------------------------------------------------------------

struct FetchResult {
    class_name: &'static str,
    start_ms: i64,
    end_ms: i64,
    events: Vec<(i64, &'static str)>,
}

async fn fetch_from_log(
    cls: &'static EventClass,
    start_ms: i64,
    end_ms: i64,
) -> anyhow::Result<FetchResult> {
    // `log show` expects UTC ISO timestamps
    let fmt = "%Y-%m-%d %H:%M:%S";
    let start = chrono::DateTime::from_timestamp_millis(start_ms)
        .context("invalid start timestamp")?
        .format(fmt)
        .to_string();
    let end = chrono::DateTime::from_timestamp_millis(end_ms)
        .context("invalid end timestamp")?
        .format(fmt)
        .to_string();

    let output = Command::new("log")
        .args([
            "show",
            "--start",
            &start,
            "--end",
            &end,
            "--predicate",
            cls.predicate,
            "--style",
            "compact",
        ])
        .output()
        .await
        .with_context(|| format!("failed to spawn 'log show' for class '{}'", cls.name))?;

    if !output.status.success() && !output.stderr.is_empty() {
        // Non-fatal — log show sometimes exits non-zero when no events
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut events = Vec::new();
    for line in stdout.lines() {
        if let Some((ts, kind)) = parse_line(cls.name, line) {
            events.push((ts, kind));
        }
    }

    Ok(FetchResult {
        class_name: cls.name,
        start_ms,
        end_ms,
        events,
    })
}

// ---------------------------------------------------------------------------
// Write results to DB
// ---------------------------------------------------------------------------

fn write_results(db: &Connection, results: Vec<FetchResult>) -> anyhow::Result<()> {
    let tx = db.unchecked_transaction()?;
    {
        let mut ins =
            tx.prepare_cached("INSERT OR IGNORE INTO events (timestamp, kind) VALUES (?1, ?2)")?;
        for r in &results {
            for (ts, kind) in &r.events {
                ins.execute(rusqlite::params![*ts, *kind])?;
            }
            update_coverage(&tx, r.class_name, r.start_ms, r.end_ms)?;
        }
    }
    tx.commit()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Opens the cache, ensures coverage for `[needed_since_ms, now]`, returns DB.
pub async fn open_unified_log_db(
    cache_path: &Utf8Path,
    needed_since_ms: i64,
) -> anyhow::Result<Connection> {
    if let Some(parent) = cache_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create cache dir {parent}"))?;
    }

    let db = Connection::open(cache_path.as_std_path())
        .with_context(|| format!("failed to open unified-log cache at {cache_path}"))?;
    ensure_schema(&db)?;

    let now = chrono::Utc::now().timestamp_millis();
    const TOLERANCE_MS: i64 = 5 * 60_000;

    // Collect fetch tasks
    let mut fetches: Vec<tokio::task::JoinHandle<anyhow::Result<FetchResult>>> = Vec::new();

    for cls in EVENT_CLASSES {
        let cov = get_coverage(&db, cls.name)?;
        match cov {
            None => {
                let start = needed_since_ms;
                let end = now;
                fetches.push(tokio::spawn(fetch_from_log(cls, start, end)));
            }
            Some((earliest, latest)) => {
                if latest < now - TOLERANCE_MS {
                    fetches.push(tokio::spawn(fetch_from_log(cls, latest, now)));
                }
                if earliest > needed_since_ms {
                    fetches.push(tokio::spawn(fetch_from_log(cls, needed_since_ms, earliest)));
                }
            }
        }
    }

    if !fetches.is_empty() {
        let mut results = Vec::with_capacity(fetches.len());
        for handle in fetches {
            let r = handle.await.context("log show task panicked")??;
            results.push(r);
        }
        write_results(&db, results)?;
    }

    Ok(db)
}

pub fn read_screen_events(db: &Connection, start_ms: i64, end_ms: i64) -> Vec<ScreenEvent> {
    let mut stmt = db
        .prepare_cached(
            "SELECT timestamp, kind FROM events \
             WHERE kind IN ('screen_lock', 'screen_unlock') \
             AND timestamp >= ?1 AND timestamp <= ?2 \
             ORDER BY timestamp",
        )
        .expect("failed to prepare screen_events query");

    stmt.query_map([start_ms, end_ms], |row| {
        let time_ms: i64 = row.get(0)?;
        let kind_str: String = row.get(1)?;
        Ok((time_ms, kind_str))
    })
    .expect("failed to query screen events")
    .filter_map(|r| r.ok())
    .filter_map(|(time_ms, kind_str)| {
        let kind = match kind_str.as_str() {
            "screen_lock" => ScreenEventKind::Lock,
            "screen_unlock" => ScreenEventKind::Unlock,
            _ => return None,
        };
        Some(ScreenEvent { time_ms, kind })
    })
    .collect()
}

pub fn read_input_events(db: &Connection, start_ms: i64, end_ms: i64) -> Vec<InputEvent> {
    let mut stmt = db
        .prepare_cached(
            "SELECT timestamp, kind FROM events \
             WHERE kind IN ('kbd', 'app_launch') \
             AND timestamp >= ?1 AND timestamp <= ?2 \
             ORDER BY timestamp",
        )
        .expect("failed to prepare input_events query");

    stmt.query_map([start_ms, end_ms], |row| {
        let time_ms: i64 = row.get(0)?;
        let kind_str: String = row.get(1)?;
        Ok((time_ms, kind_str))
    })
    .expect("failed to query input events")
    .filter_map(|r| r.ok())
    .filter_map(|(time_ms, kind_str)| {
        let kind = match kind_str.as_str() {
            "kbd" => InputEventKind::Kbd,
            "app_launch" => InputEventKind::AppLaunch,
            _ => return None,
        };
        Some(InputEvent { time_ms, kind })
    })
    .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_kbd() {
        let line =
            "2024-03-15 10:30:45.123456+0900 0x1234  Default   0x0  12345  0  SkyLight: some event";
        let result = parse_line("keyboard", line);
        assert!(result.is_some());
        let (ts, kind) = result.unwrap();
        assert_eq!(kind, "kbd");
        // Verify we got a plausible timestamp (2024-03-15 in local time)
        assert!(ts > 0);
    }

    #[test]
    fn parse_line_too_short() {
        assert!(parse_line("keyboard", "short").is_none());
    }

    #[test]
    fn parse_line_screen_lock() {
        let line = "2024-03-15 10:30:45.000000+0000 loginwindow: sendDistributedNotification: com.apple.screenIsLocked";
        let result = parse_line("screen_lock", line);
        assert!(result.is_some());
        let (_, kind) = result.unwrap();
        assert_eq!(kind, "screen_lock");
    }

    #[test]
    fn parse_line_screen_unlock() {
        let line = "2024-03-15 10:30:45.000000+0000 loginwindow: sendDistributedNotification: com.apple.screenIsUnlocked";
        let result = parse_line("screen_lock", line);
        assert!(result.is_some());
        let (_, kind) = result.unwrap();
        assert_eq!(kind, "screen_unlock");
    }
}
