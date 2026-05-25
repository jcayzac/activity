//! Imports macOS powerlog events into a persistent SQLite cache.
//!
//! Archives (`powerlog_YYYY-MM-DD_XXXXXXXX.PLSQL.gz`) are decompressed via
//! `gunzip -c` spawned as a subprocess, written to a temp file, imported, then
//! deleted immediately.

use anyhow::{Context as _, bail};
use camino::Utf8Path;
use rusqlite::Connection;
use tokio::process::Command;

const POWERLOG_DIR: &str = "/private/var/db/powerlog/Library/BatteryLife";
const LIVE_TTL_MS: i64 = 5 * 60_000;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

pub use timeline::BlEvent;
pub use timeline::PointFocusEvent as FocusEvent;

pub struct AggScreenOn {
    pub time_ms: i64,
    pub screen_on_secs: i64,
}

// ---------------------------------------------------------------------------
// Cache schema
// ---------------------------------------------------------------------------

fn ensure_schema(db: &Connection) -> anyhow::Result<()> {
    db.execute_batch(
        "PRAGMA journal_mode=WAL;
         CREATE TABLE IF NOT EXISTS bl_events (
             time   INTEGER NOT NULL PRIMARY KEY,
             active INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS focus_events (
             time      INTEGER NOT NULL PRIMARY KEY,
             bundle_id TEXT    NOT NULL
         );
         CREATE TABLE IF NOT EXISTS aggregate_screen_on (
             time      INTEGER NOT NULL PRIMARY KEY,
             screen_on INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS imported_archives (
             key TEXT NOT NULL PRIMARY KEY
         );
         CREATE TABLE IF NOT EXISTS live_coverage (
             imported_at INTEGER NOT NULL PRIMARY KEY
         );",
    )
    .context("failed to initialise powerlog cache schema")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn get_live_imported_at(db: &Connection) -> anyhow::Result<Option<i64>> {
    let mut stmt = db.prepare_cached("SELECT imported_at FROM live_coverage LIMIT 1")?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

fn set_live_imported_at(db: &Connection, ts: i64) -> anyhow::Result<()> {
    db.execute_batch("DELETE FROM live_coverage")?;
    db.execute("INSERT INTO live_coverage (imported_at) VALUES (?1)", [ts])?;
    Ok(())
}

fn is_archive_imported(db: &Connection, key: &str) -> anyhow::Result<bool> {
    let mut stmt = db.prepare_cached("SELECT 1 FROM imported_archives WHERE key = ?1")?;
    let mut rows = stmt.query([key])?;
    Ok(rows.next()?.is_some())
}

fn mark_archive_imported(db: &Connection, key: &str) -> anyhow::Result<()> {
    db.execute(
        "INSERT OR IGNORE INTO imported_archives (key) VALUES (?1)",
        [key],
    )?;
    Ok(())
}

/// Extracts `YYYY-MM-DD_XXXXXXXX` from `powerlog_YYYY-MM-DD_XXXXXXXX.PLSQL.gz`.
fn archive_key(filename: &str) -> Option<&str> {
    let stem = filename.strip_suffix(".PLSQL.gz")?;
    let key = stem.strip_prefix("powerlog_")?;
    // Validate rough shape: "YYYY-MM-DD_HEXHEX"
    if key.len() < 10 {
        return None;
    }
    Some(key)
}

// ---------------------------------------------------------------------------
// Import from a PLSQL (SQLite) file
// ---------------------------------------------------------------------------

fn import_plsql(db: &Connection, db_path: &str) -> anyhow::Result<()> {
    let src = match Connection::open_with_flags(
        db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(c) => c,
        Err(e) if e.sqlite_error_code() == Some(rusqlite::ErrorCode::CannotOpen) => {
            return Ok(());
        }
        Err(e) => {
            return Err(e).with_context(|| format!("failed to open powerlog source db {db_path}"));
        }
    };

    // Read BL events
    let bl_rows: Vec<(i64, bool)> = {
        let mut stmt = src.prepare(
            "SELECT timestamp, Active \
             FROM PLDisplayAgent_EventPoint_Display \
             WHERE Block = 'Backlight' \
             ORDER BY timestamp",
        )?;
        stmt.query_map([], |row| {
            let ts: f64 = row.get(0)?;
            let active: i64 = row.get(1)?;
            Ok(((ts * 1000.0) as i64, active == 1))
        })?
        .collect::<rusqlite::Result<_>>()?
    };

    // Read focus events
    let focus_rows: Vec<(i64, String)> = {
        let mut stmt = src.prepare(
            "SELECT timestamp, BundleID \
             FROM PLApplicationAgent_EventForward_FrontmostApp \
             ORDER BY timestamp",
        )?;
        stmt.query_map([], |row| {
            let ts: f64 = row.get(0)?;
            let bundle: String = row.get::<_, Option<String>>(1)?.unwrap_or_default();
            Ok(((ts * 1000.0) as i64, bundle))
        })?
        .collect::<rusqlite::Result<_>>()?
    };

    let tx = db.unchecked_transaction()?;
    {
        let mut ins_bl =
            tx.prepare_cached("INSERT OR IGNORE INTO bl_events (time, active) VALUES (?1, ?2)")?;
        for (time_ms, active) in &bl_rows {
            ins_bl.execute([*time_ms, if *active { 1 } else { 0 }])?;
        }
        let mut ins_focus = tx.prepare_cached(
            "INSERT OR IGNORE INTO focus_events (time, bundle_id) VALUES (?1, ?2)",
        )?;
        for (time_ms, bundle) in &focus_rows {
            ins_focus.execute(rusqlite::params![*time_ms, bundle])?;
        }
    }
    tx.commit()?;
    Ok(())
}

fn import_aggregate_live(db: &Connection) -> anyhow::Result<()> {
    let live_path = format!("{POWERLOG_DIR}/CurrentPowerlog.PLSQL");
    let src = match Connection::open_with_flags(
        &live_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(c) => c,
        Err(e) if e.sqlite_error_code() == Some(rusqlite::ErrorCode::CannotOpen) => {
            return Ok(());
        }
        Err(e) => {
            return Err(e).context("failed to open live powerlog for aggregate import");
        }
    };

    let rows: Vec<(i64, i64)> = {
        let mut stmt = src.prepare(
            "SELECT timestamp, ScreenOn \
             FROM PLDisplayAgent_Aggregate_ScreenOn \
             ORDER BY timestamp",
        )?;
        stmt.query_map([], |row| {
            let ts: f64 = row.get(0)?;
            let screen_on: i64 = row.get(1)?;
            Ok(((ts * 1000.0) as i64, screen_on))
        })?
        .collect::<rusqlite::Result<_>>()?
    };

    let tx = db.unchecked_transaction()?;
    {
        let mut ins = tx.prepare_cached(
            "INSERT OR IGNORE INTO aggregate_screen_on (time, screen_on) VALUES (?1, ?2)",
        )?;
        for (time_ms, screen_on) in &rows {
            ins.execute([*time_ms, *screen_on])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Archive decompression
// ---------------------------------------------------------------------------

async fn decompress_archive(src: &str, key: &str) -> anyhow::Result<String> {
    let tmp_dir = {
        let base = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string());
        format!("{base}activity/")
    };
    tokio::fs::create_dir_all(&tmp_dir)
        .await
        .context("failed to create temp dir")?;

    let safe_name = key.replace(['/', '\\'], "_");
    let dest = format!("{tmp_dir}{safe_name}.PLSQL");
    let tmp = format!("{dest}.tmp");

    let output = Command::new("gunzip")
        .args(["-c", src])
        .output()
        .await
        .context("failed to spawn gunzip")?;

    if !output.status.success() {
        bail!(
            "gunzip failed for {src}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    tokio::fs::write(&tmp, &output.stdout)
        .await
        .with_context(|| format!("failed to write decompressed archive to {tmp}"))?;

    tokio::fs::rename(&tmp, &dest)
        .await
        .with_context(|| format!("failed to rename {tmp} to {dest}"))?;

    Ok(dest)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Opens (or creates+populates) the powerlog cache DB.
///
/// * Imports `CurrentPowerlog.PLSQL` if stale (TTL: 5 min).
/// * Imports any not-yet-imported archive `.PLSQL.gz` files whose date >= cutoff.
/// * `cutoff = needed_since_ms - 2 * 86400 * 1000`.
pub async fn open_powerlog_db(
    cache_path: &Utf8Path,
    needed_since_ms: i64,
) -> anyhow::Result<Connection> {
    // Ensure parent directory exists
    if let Some(parent) = cache_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create cache dir {parent}"))?;
    }

    let live_path = format!("{POWERLOG_DIR}/CurrentPowerlog.PLSQL");
    let archives_dir = format!("{POWERLOG_DIR}/Archives");

    // Check accessibility before opening cache
    match std::fs::metadata(POWERLOG_DIR) {
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            bail!(
                "Powerlog cannot be accessed. \
                 Grant Full Disk Access to your terminal in System Settings."
            );
        }
        _ => {}
    }

    let db = Connection::open(cache_path.as_std_path())
        .with_context(|| format!("failed to open powerlog cache at {cache_path}"))?;
    ensure_schema(&db)?;

    let now = chrono::Utc::now().timestamp_millis();
    let cutoff = needed_since_ms - 2 * 86_400_000;

    // ---- Live DB ----
    let live_imported_at = get_live_imported_at(&db)?;
    let needs_live_import = live_imported_at
        .map(|t| now - t >= LIVE_TTL_MS)
        .unwrap_or(true);

    if needs_live_import {
        match std::fs::metadata(&live_path) {
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                bail!(
                    "Powerlog cannot be accessed. \
                     Grant Full Disk Access to your terminal in System Settings."
                );
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e).context("unexpected error accessing powerlog"),
            Ok(_) => {
                import_plsql(&db, &live_path)?;
                import_aggregate_live(&db)?;
            }
        }
        set_live_imported_at(&db, now)?;
    }

    // ---- Archives ----
    let mut to_import: Vec<(String, String)> = Vec::new(); // (src_path, key)

    match std::fs::read_dir(&archives_dir) {
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            bail!(
                "Powerlog cannot be accessed. \
                 Grant Full Disk Access to your terminal in System Settings."
            );
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e).context("unexpected error reading powerlog archives dir"),
        Ok(entries) => {
            for entry in entries {
                let entry = entry.context("failed to read archive dir entry")?;
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if !name.ends_with(".PLSQL.gz") {
                    continue;
                }
                let Some(key) = archive_key(&name) else {
                    continue;
                };
                // Parse archive date from key prefix YYYY-MM-DD
                let date_str = &key[..10];
                let archive_date_ms = match chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                {
                    Ok(d) => d
                        .and_hms_opt(0, 0, 0)
                        .map(|dt| dt.and_utc().timestamp_millis())
                        .unwrap_or(0),
                    Err(_) => continue,
                };
                if archive_date_ms < cutoff {
                    continue;
                }
                if is_archive_imported(&db, key)? {
                    continue;
                }
                let src = format!("{archives_dir}/{name}");
                to_import.push((src, key.to_string()));
            }
        }
    }

    // Decompress all archives in parallel
    let decompress_futures: Vec<_> = to_import
        .iter()
        .map(|(src, key)| {
            let src = src.clone();
            let key = key.clone();
            tokio::spawn(async move { decompress_archive(&src, &key).await.map(|p| (p, key)) })
        })
        .collect();

    let mut decompressed: Vec<(String, String)> = Vec::new();
    for handle in decompress_futures {
        let result = handle
            .await
            .context("archive decompression task panicked")??;
        decompressed.push(result);
    }

    // Import decompressed archives and clean up
    for (dest, key) in &decompressed {
        let import_result = import_plsql(&db, dest);
        // Always try to remove the temp file
        let _ = std::fs::remove_file(dest);
        import_result.with_context(|| format!("failed to import archive {key}"))?;
        mark_archive_imported(&db, key)?;
    }

    Ok(db)
}

pub fn all_bl_events(db: &Connection) -> Vec<BlEvent> {
    let mut stmt = db
        .prepare_cached("SELECT time, active FROM bl_events ORDER BY time")
        .expect("failed to prepare bl_events query");
    stmt.query_map([], |row| {
        Ok(BlEvent {
            time_ms: row.get(0)?,
            active: row.get::<_, i64>(1)? == 1,
        })
    })
    .expect("failed to query bl_events")
    .filter_map(|r| r.ok())
    .collect()
}

pub fn all_focus_events(db: &Connection) -> Vec<FocusEvent> {
    let mut stmt = db
        .prepare_cached("SELECT time, bundle_id FROM focus_events ORDER BY time")
        .expect("failed to prepare focus_events query");
    stmt.query_map([], |row| {
        Ok(FocusEvent {
            time_ms: row.get(0)?,
            bundle_id: row.get(1)?,
        })
    })
    .expect("failed to query focus_events")
    .filter_map(|r| r.ok())
    .collect()
}

pub fn aggregate_screen_on(db: &Connection, start_ms: i64, end_ms: i64) -> Vec<AggScreenOn> {
    let mut stmt = db
        .prepare_cached(
            "SELECT time, screen_on FROM aggregate_screen_on \
             WHERE time >= ?1 AND time <= ?2 ORDER BY time",
        )
        .expect("failed to prepare aggregate_screen_on query");
    stmt.query_map([start_ms, end_ms], |row| {
        Ok(AggScreenOn {
            time_ms: row.get(0)?,
            screen_on_secs: row.get(1)?,
        })
    })
    .expect("failed to query aggregate_screen_on")
    .filter_map(|r| r.ok())
    .collect()
}
