//! Reads /app/usage sessions from macOS knowledgeC.db (Screen Time / Knowledge store)
//! and imports them into a persistent SQLite cache as focus_periods.
//!
//! knowledgeC.db is located at ~/Library/Application Support/Knowledge/knowledgeC.db
//! and is readable without Full Disk Access.

use anyhow::Context as _;
use camino::Utf8Path;
use rusqlite::Connection;

use crate::SourcesError;

// ---------------------------------------------------------------------------
// Open knowledge cache DB
// ---------------------------------------------------------------------------

/// Opens (or creates) the knowledge cache DB at `path`.
///
/// The DB holds `focus_periods` and `knowledge_coverage` — data sourced
/// from knowledgeC.db.
/// Opens (or creates) the knowledge cache DB at `path`.
///
/// The DB holds `focus_periods` and `knowledge_coverage` — data sourced
/// from knowledgeC.db.
///
/// # Errors
///
/// Returns an error if the directory cannot be created or the DB cannot be opened.
pub fn open_knowledge_db(path: &Utf8Path) -> Result<Connection, SourcesError> {
    use anyhow::Context as _;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create cache dir {parent}"))?;
    }
    let conn = Connection::open(path.as_std_path())
        .with_context(|| format!("failed to open knowledge cache db at {path}"))?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;
         CREATE TABLE IF NOT EXISTS focus_periods (
             first     INTEGER NOT NULL,
             last      INTEGER NOT NULL,
             bundle_id TEXT    NOT NULL,
             PRIMARY KEY (first, bundle_id)
         );
         CREATE TABLE IF NOT EXISTS knowledge_coverage (
             imported_through INTEGER NOT NULL PRIMARY KEY
         );",
    )
    .with_context(|| "failed to initialize knowledge cache schema")?;
    Ok(conn)
}

/// Cocoa epoch offset in seconds (2001-01-01 00:00:00 UTC as Unix timestamp).
const COCOA_EPOCH_OFFSET_S: f64 = 978_307_200.0;

fn cocoa_to_ms(cocoa: f64) -> i64 {
    ((COCOA_EPOCH_OFFSET_S + cocoa) * 1000.0) as i64
}

fn ms_to_cocoa(ms: i64) -> f64 {
    (ms as f64 / 1000.0) - COCOA_EPOCH_OFFSET_S
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

pub use timeline::FocusPeriod;

// ---------------------------------------------------------------------------
// Schema helpers
// ---------------------------------------------------------------------------


pub fn knowledge_coverage(db: &Connection) -> Result<i64, SourcesError> {
    let mut stmt = db
        .prepare_cached("SELECT imported_through FROM knowledge_coverage LIMIT 1")?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        Ok(row.get(0).unwrap_or(0))
    } else {
        Ok(0)
    }
}

fn set_knowledge_coverage(db: &Connection, ms: i64) -> Result<(), SourcesError> {
    db.execute_batch("DELETE FROM knowledge_coverage")?;
    db.execute(
        "INSERT INTO knowledge_coverage (imported_through) VALUES (?1)",
        [ms],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Imports `/app/usage` sessions from `knowledgeC.db` that end after `since_ms`.
///
/// Returns the count of newly inserted rows.  A missing `knowledgeC.db` is
/// silently ignored (returns 0).
/// Imports `/app/usage` sessions from `knowledgeC.db` that end after `since_ms`.
///
/// Returns the count of newly inserted rows. A missing or inaccessible
/// `knowledgeC.db` is silently ignored (returns 0).
///
/// # Errors
///
/// Returns an error if the cache DB operations fail.
pub fn import_knowledge_focus_periods(
    db: &Connection,
    knowledge_db_path: &Utf8Path,
    since_ms: i64,
) -> Result<i64, SourcesError> {
    let kdb = match Connection::open_with_flags(
        knowledge_db_path.as_std_path(),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(c) => c,
        Err(e) if e.sqlite_error_code() == Some(rusqlite::ErrorCode::CannotOpen) => {
            return Ok(0);
        }
        Err(e) => {
            return Err(anyhow::Error::from(e)
                .context(format!("failed to open knowledgeC.db at {knowledge_db_path}"))
                .into());
        }
    };

    let since_cocoa = ms_to_cocoa(since_ms);

    let rows: Vec<(String, f64, f64)> = {
        let mut stmt = kdb.prepare(
            "SELECT ZVALUESTRING, ZSTARTDATE, ZENDDATE \
             FROM ZOBJECT \
             WHERE ZSTREAMNAME = '/app/usage' \
               AND ZENDDATE > ?1 \
               AND ZVALUESTRING IS NOT NULL \
             ORDER BY ZSTARTDATE",
        )?;
        stmt.query_map([since_cocoa], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, f64>(2)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()
        .context("failed to query knowledgeC.db")?
    };

    if rows.is_empty() {
        return Ok(0);
    }

    let tx = db.unchecked_transaction()?;
    let mut inserted: i64 = 0;
    {
        let mut ins = tx.prepare_cached(
            "INSERT OR IGNORE INTO focus_periods (first, last, bundle_id) VALUES (?1, ?2, ?3)",
        )?;
        for (bundle_id, start_cocoa, end_cocoa) in &rows {
            let first = cocoa_to_ms(*start_cocoa);
            let last = cocoa_to_ms(*end_cocoa);
            if last <= first {
                continue;
            }
            let changes = ins.execute(rusqlite::params![first, last, bundle_id])?;
            inserted += changes as i64;
        }
    }
    tx.commit()?;

    // Advance coverage watermark
    if let Some((_, end_cocoa, _)) = rows.last() {
        let latest_ms = cocoa_to_ms(*end_cocoa);
        set_knowledge_coverage(db, latest_ms)?;
    }

    Ok(inserted)
}

pub fn all_focus_periods(db: &Connection) -> Result<Vec<FocusPeriod>, SourcesError> {
    let mut stmt = db
        .prepare_cached("SELECT first, last, bundle_id FROM focus_periods ORDER BY first")?;
    let periods = stmt
        .query_map([], |row| {
            Ok(FocusPeriod {
                first_ms: row.get(0)?,
                last_ms: row.get(1)?,
                bundle_id: row.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(periods)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{cocoa_to_ms, ms_to_cocoa};

    #[test]
    fn cocoa_to_ms_known_value() {
        // Cocoa epoch itself (0.0) should be 2001-01-01 00:00:00 UTC
        // = 978_307_200_000 ms since Unix epoch
        let expected = 978_307_200_000_i64;
        assert_eq!(cocoa_to_ms(0.0), expected);
    }

    #[test]
    fn cocoa_round_trip() {
        let ms: i64 = 1_700_000_000_000; // some timestamp in 2023
        let cocoa = ms_to_cocoa(ms);
        let back = cocoa_to_ms(cocoa);
        // Allow 1 ms rounding error
        assert!(
            (back - ms).abs() <= 1,
            "round-trip drift: {}",
            (back - ms).abs()
        );
    }
}
