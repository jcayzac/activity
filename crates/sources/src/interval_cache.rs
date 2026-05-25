// Persistence layer for the interval cache (active-on.db).
// Ported from lib/interval-cache.ts.

use rusqlite::Connection;

use timeline::date_utils::effective_day;
use timeline::{Interval, IntervalLabel};

// ---------------------------------------------------------------------------
// Open DB
// ---------------------------------------------------------------------------

pub fn open_interval_cache_db(path: &camino::Utf8Path) -> anyhow::Result<Connection> {
    use anyhow::Context as _;
    let conn = Connection::open(path.as_std_path())
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
    Ok(conn)
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

pub struct CachedIntervals {
    pub provisional: bool,
    pub intervals: Vec<Interval>,
}

// ---------------------------------------------------------------------------
// Read
// ---------------------------------------------------------------------------

pub fn read_cached_intervals(db: &Connection, date: &str) -> Option<CachedIntervals> {
    let provisional: i64 = db
        .query_row(
            "SELECT provisional FROM interval_cache WHERE date = ?1",
            rusqlite::params![date],
            |row| row.get(0),
        )
        .ok()?;

    let mut stmt = db
        .prepare("SELECT first, last, label FROM intervals WHERE date = ?1 ORDER BY first")
        .expect("read_cached_intervals: prepare failed");

    let intervals: Vec<Interval> = stmt
        .query_map(rusqlite::params![date], |row| {
            let first_ms: i64 = row.get(0)?;
            let last_ms: i64 = row.get(1)?;
            let label_str: String = row.get(2)?;
            Ok((first_ms, last_ms, label_str))
        })
        .expect("read_cached_intervals: query failed")
        .filter_map(|r| r.ok())
        .filter_map(|(first_ms, last_ms, label_str)| {
            let label = match label_str.as_str() {
                "active" => IntervalLabel::Active,
                "break" => IntervalLabel::Break,
                "transit" => IntervalLabel::Transit,
                _ => return None,
            };
            Some(Interval {
                first_ms,
                last_ms,
                label,
                location: None,
            })
        })
        .collect();

    Some(CachedIntervals {
        provisional: provisional == 1,
        intervals,
    })
}

// ---------------------------------------------------------------------------
// Write
// ---------------------------------------------------------------------------

pub fn write_cached_intervals(
    db: &Connection,
    date: &str,
    intervals: &[Interval],
    provisional: bool,
) {
    db.execute(
        "INSERT INTO interval_cache (date, provisional) VALUES (?1, ?2)
         ON CONFLICT (date) DO UPDATE SET provisional = excluded.provisional",
        rusqlite::params![date, if provisional { 1i64 } else { 0i64 }],
    )
    .expect("write_cached_intervals: upsert meta failed");

    db.execute(
        "DELETE FROM intervals WHERE date = ?1",
        rusqlite::params![date],
    )
    .expect("write_cached_intervals: delete failed");

    let mut stmt = db
        .prepare("INSERT INTO intervals (date, first, last, label) VALUES (?1, ?2, ?3, ?4)")
        .expect("write_cached_intervals: prepare insert failed");

    for iv in intervals {
        let label_str = match iv.label {
            IntervalLabel::Active => "active",
            IntervalLabel::Break => "break",
            IntervalLabel::Transit => "transit",
        };
        stmt.execute(rusqlite::params![date, iv.first_ms, iv.last_ms, label_str])
            .expect("write_cached_intervals: insert failed");
    }
}

// ---------------------------------------------------------------------------
// Cache reuse decision
// ---------------------------------------------------------------------------

/// Returns `true` when the cached data for `date` can be used without
/// recomputing.  Mirrors `shouldReuseCache` in lib/interval-cache.ts.
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
    use super::{read_cached_intervals, should_reuse_cache, write_cached_intervals};
    use timeline::date_utils::six_am_of;
    use timeline::{Interval, IntervalLabel};

    #[test]
    fn should_reuse_cache_future_date() {
        assert!(!should_reuse_cache("2099-01-01", "2024-01-01"));
    }

    #[test]
    fn should_reuse_cache_today() {
        // today == date => false
        assert!(!should_reuse_cache("2024-03-15", "2024-03-15"));
    }

    #[test]
    fn should_reuse_cache_tomorrow() {
        assert!(!should_reuse_cache("2099-12-31", "2024-03-15"));
    }

    #[test]
    fn should_reuse_cache_old_past_date() {
        // A date safely in the past relative to "now" — not the effective day.
        // We use "1970-01-01" which is definitely not the effective day.
        assert!(should_reuse_cache("1970-01-01", "2099-12-31"));
    }

    #[test]
    fn round_trip_write_read() {
        let db = rusqlite::Connection::open_in_memory().unwrap();
        db.execute_batch(
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

        write_cached_intervals(&db, date, &ivs, false);
        let cached = read_cached_intervals(&db, date).unwrap();
        assert!(!cached.provisional);
        assert_eq!(cached.intervals.len(), 2);
        assert_eq!(cached.intervals[0].first_ms, six_am);
        assert_eq!(cached.intervals[0].label, IntervalLabel::Active);
        assert_eq!(cached.intervals[1].label, IntervalLabel::Break);
    }
}
