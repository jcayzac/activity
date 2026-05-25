//! Imports macOS Biome App.InFocus events into a persistent SQLite cache.
//!
//! Only gained-focus events (field 3 = 1) are imported — they are direct
//! equivalents of powerlog focus_events point events.

use camino::Utf8Path;
use rusqlite::Connection;

use crate::SourcesError;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

pub use timeline::PointFocusEvent as InFocusEvent;

// ---------------------------------------------------------------------------
// Open infocus cache DB
// ---------------------------------------------------------------------------

/// Opens (or creates) the InFocus cache DB at `path`.
///
/// The DB holds `infocus_events` and `infocus_coverage` — data sourced
/// from Biome App.InFocus stream files.
/// Opens (or creates) the InFocus cache DB at `path`.
///
/// The DB holds `infocus_events` and `infocus_coverage` — data sourced
/// from Biome App.InFocus stream files.
///
/// # Errors
///
/// Returns an error if the directory cannot be created or the DB cannot be opened.
pub fn open_infocus_db(path: &Utf8Path) -> Result<Connection, SourcesError> {
    use anyhow::Context as _;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create cache dir {parent}"))?;
    }
    let conn = Connection::open(path.as_std_path())
        .with_context(|| format!("failed to open infocus cache db at {path}"))?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;
         CREATE TABLE IF NOT EXISTS infocus_events (
             time      INTEGER NOT NULL PRIMARY KEY,
             bundle_id TEXT    NOT NULL
         );
         CREATE TABLE IF NOT EXISTS infocus_coverage (
             imported_through INTEGER NOT NULL PRIMARY KEY
         );",
    )
    .with_context(|| "failed to initialize infocus cache schema")?;
    Ok(conn)
}


pub fn infocus_coverage(db: &Connection) -> Result<i64, SourcesError> {
    let mut stmt = db
        .prepare_cached("SELECT imported_through FROM infocus_coverage LIMIT 1")?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        Ok(row.get(0).unwrap_or(0))
    } else {
        Ok(0)
    }
}

fn set_infocus_coverage(db: &Connection, ms: i64) -> Result<(), SourcesError> {
    db.execute_batch("DELETE FROM infocus_coverage")?;
    db.execute(
        "INSERT INTO infocus_coverage (imported_through) VALUES (?1)",
        [ms],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Imports App.InFocus gained-focus events from Biome stream files.
///
/// Processes all numeric-named files in `stream_dir`, inserting only events
/// with `time_ms > since_ms`. Returns the number of newly inserted events.
/// Imports App.InFocus gained-focus events from Biome stream files.
///
/// Processes all numeric-named files in `stream_dir`, inserting only events
/// with `time_ms > since_ms`. Returns the number of newly inserted events.
///
/// # Errors
///
/// Returns an error if the stream directory cannot be read or DB operations fail.
pub async fn import_infocus_events(
    db: &Connection,
    stream_dir: &Utf8Path,
    since_ms: i64,
) -> Result<i64, SourcesError> {
    // Collect stream files (numeric names only, sorted)
    let mut stream_files: Vec<String> = Vec::new();
    match tokio::fs::read_dir(stream_dir.as_std_path()).await {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {}
        Err(e) => {
            return Err(anyhow::Error::from(e)
                .context(format!("failed to read infocus stream dir {stream_dir}"))
                .into());
        }
        Ok(mut entries) => {
            while let Some(entry) = entries.next_entry().await? {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name.chars().all(|c| c.is_ascii_digit()) {
                    stream_files.push(entry.path().to_string_lossy().into_owned());
                }
            }
        }
    }
    stream_files.sort();

    let mut latest_ms = since_ms;
    let mut inserted: i64 = 0;

    let tx = db.unchecked_transaction()?;
    {
        let mut ins = tx.prepare_cached(
            "INSERT OR IGNORE INTO infocus_events (time, bundle_id) VALUES (?1, ?2)",
        )?;

        for path_str in &stream_files {
            let path = camino::Utf8Path::new(path_str);
            let iter = match proto::segb::iter_records(path) {
                Ok(it) => it,
                Err(_) => continue,
            };
            for payload in iter {
                let Some(ev) = proto::parse_infocus_record(&payload) else {
                    continue;
                };
                if ev.time_ms <= since_ms {
                    continue;
                }
                let changes = ins.execute(rusqlite::params![ev.time_ms, ev.bundle_id])?;
                inserted += changes as i64;
                if ev.time_ms > latest_ms {
                    latest_ms = ev.time_ms;
                }
            }
        }
    }
    tx.commit()?;

    if latest_ms > since_ms {
        set_infocus_coverage(db, latest_ms)?;
    }

    Ok(inserted)
}

pub fn all_infocus_events(db: &Connection) -> Result<Vec<InFocusEvent>, SourcesError> {
    let mut stmt = db
        .prepare_cached("SELECT time, bundle_id FROM infocus_events ORDER BY time")?;
    let events = stmt
        .query_map([], |row| {
            Ok(InFocusEvent {
                time_ms: row.get(0)?,
                bundle_id: row.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(events)
}
