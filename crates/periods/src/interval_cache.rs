// Persistence layer for the interval cache (active-on.db).
// Ported from lib/interval-cache.ts.

use camino::Utf8Path;

use timeline::date_utils::effective_day;
use timeline::{Interval, IntervalLabel};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum IntervalCacheError {
    #[error(transparent)]
    Db(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// ---------------------------------------------------------------------------
// Opaque DB handle
// ---------------------------------------------------------------------------

/// An open connection to the interval cache DB.
///
/// Wraps `rusqlite::Connection` so the storage engine does not appear in the
/// public API of the use-case layer.
pub struct IntervalCacheDb(rusqlite::Connection);

// ---------------------------------------------------------------------------
// Open DB
// ---------------------------------------------------------------------------

/// Opens (or creates) the interval cache DB at `path`.
pub fn open_interval_cache_db(path: &Utf8Path) -> Result<IntervalCacheDb, IntervalCacheError> {
    use anyhow::Context as _;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create cache dir {parent}"))?;
    }
    let conn = rusqlite::Connection::open(path.as_std_path())
        .with_context(|| format!("failed to open interval cache db at {path}"))?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;
         CREATE TABLE IF NOT EXISTS interval_cache (
             date        TEXT    NOT NULL PRIMARY KEY,
             provisional INTEGER NOT NULL DEFAULT 0
         );
         CREATE TABLE IF NOT EXISTS intervals (
             date  TEXT    NOT NULL,
             first INTEGER NOT NULL,
             last  INTEGER NOT NULL,
             label TEXT    NOT NULL,
             PRIMARY KEY (date, first)
         );",
    )
    .with_context(|| "failed to initialize interval cache schema")?;
    Ok(IntervalCacheDb(conn))
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Cached intervals for one calendar date.
#[derive(Debug, Clone)]
pub struct CachedIntervals {
    pub provisional: bool,
    pub intervals: Vec<Interval>,
}

// ---------------------------------------------------------------------------
// Read
// ---------------------------------------------------------------------------

/// Returns the cached intervals for `date`, or `None` if the date has no entry.
pub fn read_cached_intervals(
    db: &IntervalCacheDb,
    date: &str,
) -> Result<Option<CachedIntervals>, IntervalCacheError> {
    let provisional: i64 = match db.0.query_row(
        "SELECT provisional FROM interval_cache WHERE date = ?1",
        rusqlite::params![date],
        |row| row.get(0),
    ) {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(e) => return Err(e.into()),
    };

    let mut stmt = db
        .0
        .prepare("SELECT first, last, label FROM intervals WHERE date = ?1 ORDER BY first")?;

    let intervals: Vec<Interval> = stmt
        .query_map(rusqlite::params![date], |row| {
            let first_ms: i64 = row.get(0)?;
            let last_ms: i64 = row.get(1)?;
            let label_str: String = row.get(2)?;
            Ok((first_ms, last_ms, label_str))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .filter_map(|(first_ms, last_ms, label_str)| {
            let label = match label_str.as_str() {
                "active" => IntervalLabel::Active,
                "break" => IntervalLabel::Break,
                "transit" => IntervalLabel::Transit,
                _ => return None,
            };
            Some(Interval { first_ms, last_ms, label, location: None })
        })
        .collect();

    Ok(Some(CachedIntervals { provisional: provisional == 1, intervals }))
}

// ---------------------------------------------------------------------------
// Write
// ---------------------------------------------------------------------------

/// Writes (or replaces) the cached intervals for `date`.
pub fn write_cached_intervals(
    db: &IntervalCacheDb,
    date: &str,
    intervals: &[Interval],
    provisional: bool,
) -> Result<(), IntervalCacheError> {
    db.0.execute(
        "INSERT INTO interval_cache (date, provisional) VALUES (?1, ?2)
         ON CONFLICT (date) DO UPDATE SET provisional = excluded.provisional",
        rusqlite::params![date, if provisional { 1i64 } else { 0i64 }],
    )?;

    db.0.execute("DELETE FROM intervals WHERE date = ?1", rusqlite::params![date])?;

    let mut stmt = db
        .0
        .prepare("INSERT INTO intervals (date, first, last, label) VALUES (?1, ?2, ?3, ?4)")?;

    for iv in intervals {
        let label_str = match iv.label {
            IntervalLabel::Active => "active",
            IntervalLabel::Break => "break",
            IntervalLabel::Transit => "transit",
        };
        stmt.execute(rusqlite::params![date, iv.first_ms, iv.last_ms, label_str])?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Cache reuse decision
// ---------------------------------------------------------------------------

/// Returns `true` when the cached data for `date` can be used without
/// recomputing. Mirrors `shouldReuseCache` in lib/interval-cache.ts.
///
/// Returns `false` if:
/// - `date >= today`  (future or current calendar day)
/// - `date == effective_day(now)`  (still within the rolling 06:00 window)
pub fn should_reuse_cache(date: &str, today: &str) -> bool {
    if date >= today {
        return false;
    }
    let now_ms = chrono::Utc::now().timestamp_millis();
    if date == effective_day(now_ms) {
        return false;
    }
    true
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{
        CachedIntervals, IntervalCacheDb, read_cached_intervals, should_reuse_cache,
        write_cached_intervals,
    };
    use timeline::date_utils::six_am_of;
    use timeline::{Interval, IntervalLabel};

    fn make_in_memory_db() -> IntervalCacheDb {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE interval_cache (
                date        TEXT    NOT NULL PRIMARY KEY,
                provisional INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE intervals (
                date  TEXT    NOT NULL,
                first INTEGER NOT NULL,
                last  INTEGER NOT NULL,
                label TEXT    NOT NULL,
                PRIMARY KEY (date, first)
            );",
        )
        .unwrap();
        IntervalCacheDb(conn)
    }

    #[test]
    fn should_reuse_cache_future_date() {
        assert!(!should_reuse_cache("2099-01-01", "2024-01-01"));
    }

    #[test]
    fn should_reuse_cache_today() {
        assert!(!should_reuse_cache("2024-03-15", "2024-03-15"));
    }

    #[test]
    fn should_reuse_cache_tomorrow() {
        assert!(!should_reuse_cache("2099-12-31", "2024-03-15"));
    }

    #[test]
    fn should_reuse_cache_old_past_date() {
        assert!(should_reuse_cache("1970-01-01", "2099-12-31"));
    }

    #[test]
    fn round_trip_write_read() {
        let db = make_in_memory_db();
        let date = "2024-03-15";
        let six_am = six_am_of(date);
        let ivs = vec![
            Interval {
                first_ms: six_am,
                last_ms: six_am + 3_600_000,
                label: IntervalLabel::Active,
                location: None,
            },
            Interval {
                first_ms: six_am + 3_600_000,
                last_ms: six_am + 5_400_000,
                label: IntervalLabel::Break,
                location: None,
            },
        ];

        write_cached_intervals(&db, date, &ivs, false).unwrap();
        let cached: CachedIntervals = read_cached_intervals(&db, date).unwrap().unwrap();
        assert!(!cached.provisional);
        assert_eq!(cached.intervals.len(), 2);
        assert_eq!(cached.intervals[0].first_ms, six_am);
        assert_eq!(cached.intervals[0].label, IntervalLabel::Active);
        assert_eq!(cached.intervals[1].label, IntervalLabel::Break);
    }

    #[test]
    fn read_missing_date_returns_none() {
        let db = make_in_memory_db();
        let result = read_cached_intervals(&db, "2024-01-01").unwrap();
        assert!(result.is_none());
    }
}
