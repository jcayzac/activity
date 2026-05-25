//! Imports macOS Biome App.InFocus events into a persistent SQLite cache.
//!
//! Only gained-focus events (field 3 = 1) are imported — they are direct
//! equivalents of powerlog focus_events point events.

use anyhow::Context as _;
use camino::Utf8Path;
use rusqlite::Connection;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

pub use timeline::PointFocusEvent as InFocusEvent;

// ---------------------------------------------------------------------------
// Schema helpers
// ---------------------------------------------------------------------------

pub fn init_infocus_schema(db: &Connection) {
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS infocus_events (
             time      INTEGER NOT NULL PRIMARY KEY,
             bundle_id TEXT    NOT NULL
         );
         CREATE TABLE IF NOT EXISTS infocus_coverage (
             imported_through INTEGER NOT NULL PRIMARY KEY
         );",
    )
    .expect("failed to init infocus schema");
}

pub fn infocus_coverage(db: &Connection) -> i64 {
    let mut stmt = db
        .prepare_cached("SELECT imported_through FROM infocus_coverage LIMIT 1")
        .expect("failed to prepare infocus_coverage query");
    let mut rows = stmt.query([]).expect("failed to query infocus_coverage");
    if let Some(row) = rows.next().expect("row error") {
        row.get(0).unwrap_or(0)
    } else {
        0
    }
}

fn set_infocus_coverage(db: &Connection, ms: i64) {
    db.execute_batch("DELETE FROM infocus_coverage")
        .expect("failed to delete infocus_coverage");
    db.execute(
        "INSERT INTO infocus_coverage (imported_through) VALUES (?1)",
        [ms],
    )
    .expect("failed to insert infocus_coverage");
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Imports App.InFocus gained-focus events from Biome stream files.
///
/// Processes all numeric-named files in `stream_dir`, inserting only events
/// with `time_ms > since_ms`. Returns the number of newly inserted events.
pub async fn import_infocus_events(
    db: &Connection,
    stream_dir: &Utf8Path,
    since_ms: i64,
) -> anyhow::Result<i64> {
    // Collect stream files (numeric names only, sorted)
    let mut stream_files: Vec<String> = Vec::new();
    match tokio::fs::read_dir(stream_dir.as_std_path()).await {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {}
        Err(e) => {
            return Err(e)
                .with_context(|| format!("failed to read infocus stream dir {stream_dir}"));
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
        set_infocus_coverage(db, latest_ms);
    }

    Ok(inserted)
}

pub fn all_infocus_events(db: &Connection) -> Vec<InFocusEvent> {
    let mut stmt = db
        .prepare_cached("SELECT time, bundle_id FROM infocus_events ORDER BY time")
        .expect("failed to prepare infocus_events query");
    stmt.query_map([], |row| {
        Ok(InFocusEvent {
            time_ms: row.get(0)?,
            bundle_id: row.get(1)?,
        })
    })
    .expect("failed to query infocus_events")
    .filter_map(|r| r.ok())
    .collect()
}
