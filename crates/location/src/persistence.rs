use std::collections::HashMap;

use anyhow::Context as _;
use camino::Utf8Path;
use chrono::Utc;
use rusqlite::Connection;

use crate::LocationError;
use crate::algorithm::order_signals;
use crate::{
    CLUSTER_THRESHOLD_SECONDS, HALF_LIFE_DAYS, HOME_LOCATION_TYPE, MIN_OVERLAP_SECONDS,
    RTO_SCHEMA_VERSION, SignalPeriod,
};
use timeline::{RtoBlock, RtoData};

// ---------------------------------------------------------------------------
// Open RTO DB
// ---------------------------------------------------------------------------

pub fn open_rto_db(path: &Utf8Path) -> Result<Connection, LocationError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create cache dir {parent}"))?;
    }
    let conn = Connection::open(path.as_std_path())
        .with_context(|| format!("failed to open rto db at {path}"))?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;
         CREATE TABLE IF NOT EXISTS evidence_cache (
             date        TEXT    NOT NULL PRIMARY KEY,
             provisional INTEGER NOT NULL DEFAULT 0
         );
         CREATE TABLE IF NOT EXISTS evidence (
             date     TEXT    NOT NULL,
             first    INTEGER NOT NULL,
             last     INTEGER NOT NULL,
             location TEXT
         );
         CREATE INDEX IF NOT EXISTS evidence_date ON evidence (date);
         CREATE TABLE IF NOT EXISTS schema_meta (
             key   TEXT    NOT NULL PRIMARY KEY,
             value INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS co_occurrences (
             signal_a      TEXT    NOT NULL,
             signal_b      TEXT    NOT NULL,
             total_seconds REAL    NOT NULL DEFAULT 0,
             last_seen     INTEGER NOT NULL,
             PRIMARY KEY (signal_a, signal_b)
         );
         CREATE TABLE IF NOT EXISTS co_occurrence_watermark (
             id           INTEGER NOT NULL PRIMARY KEY CHECK (id = 1),
             processed_ms INTEGER NOT NULL
         );",
    )
    .with_context(|| "failed to initialize rto db schema")?;
    Ok(conn)
}

// ---------------------------------------------------------------------------
// Watermark
// ---------------------------------------------------------------------------

/// Returns the co-occurrence processing watermark (ms since Unix epoch), or 0 if not set.
pub fn watermark(db: &Connection) -> i64 {
    db.query_row(
        "SELECT processed_ms FROM co_occurrence_watermark WHERE id = 1",
        [],
        |row| row.get::<_, i64>(0),
    )
    .unwrap_or(0)
}

/// Advances the co-occurrence processing watermark to `ms`.
pub fn set_watermark(db: &Connection, ms: i64) -> Result<(), LocationError> {
    db.execute(
        "INSERT INTO co_occurrence_watermark (id, processed_ms) VALUES (1, ?)
         ON CONFLICT (id) DO UPDATE SET processed_ms = excluded.processed_ms",
        rusqlite::params![ms],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Co-occurrence update
// ---------------------------------------------------------------------------

pub fn update_co_occurrences(
    db: &Connection,
    duet_periods: &[SignalPeriod],
    subnet_periods: &[SignalPeriod],
    since_ms: i64,
    min_overlap_seconds: f64,
) -> Result<(), LocationError> {
    if duet_periods.is_empty() || subnet_periods.is_empty() {
        return Ok(());
    }

    let fresh_duet: Vec<SignalPeriod> = duet_periods
        .iter()
        .filter(|p| p.end_ms > since_ms)
        .map(|p| {
            if since_ms > p.start_ms {
                SignalPeriod { start_ms: since_ms, end_ms: p.end_ms, signal: p.signal.clone() }
            } else {
                p.clone()
            }
        })
        .collect();

    if fresh_duet.is_empty() {
        return Ok(());
    }

    struct Pair {
        a: String,
        b: String,
        seconds: f64,
        last_seen_ms: i64,
    }
    let mut pairs: Vec<Pair> = Vec::new();

    let mut si = 0usize;
    for dp in &fresh_duet {
        while si < subnet_periods.len() && subnet_periods[si].end_ms <= dp.start_ms {
            si += 1;
        }
        let mut j = si;
        while j < subnet_periods.len() {
            if subnet_periods[j].start_ms >= dp.end_ms {
                break;
            }
            let overlap_start = dp.start_ms.max(subnet_periods[j].start_ms);
            let overlap_end = dp.end_ms.min(subnet_periods[j].end_ms);
            let overlap_ms = overlap_end - overlap_start;
            if overlap_ms <= 0 {
                j += 1;
                continue;
            }
            let overlap_seconds = overlap_ms as f64 / 1000.0;
            if overlap_seconds < min_overlap_seconds {
                j += 1;
                continue;
            }
            let [a, b] = order_signals(&dp.signal, &subnet_periods[j].signal);
            pairs.push(Pair { a, b, seconds: overlap_seconds, last_seen_ms: overlap_end });
            j += 1;
        }
    }

    if pairs.is_empty() {
        return Ok(());
    }

    db.execute_batch("BEGIN")?;
    for pair in &pairs {
        db.execute(
            "INSERT INTO co_occurrences (signal_a, signal_b, total_seconds, last_seen)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (signal_a, signal_b) DO UPDATE SET
                 total_seconds = total_seconds + excluded.total_seconds,
                 last_seen = MAX(last_seen, excluded.last_seen)",
            rusqlite::params![pair.a, pair.b, pair.seconds, pair.last_seen_ms],
        )?;
    }
    db.execute_batch("COMMIT")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Cluster resolution
// ---------------------------------------------------------------------------

pub fn resolve_location_clusters(
    db: &Connection,
    half_life_days: f64,
    threshold_seconds: f64,
) -> Result<HashMap<String, String>, LocationError> {
    let now_ms = Utc::now().timestamp_millis();

    struct EdgeRow {
        signal_a: String,
        signal_b: String,
        total_seconds: f64,
        last_seen: i64,
    }

    let mut stmt = db
        .prepare("SELECT signal_a, signal_b, total_seconds, last_seen FROM co_occurrences")?;

    let mut edges: Vec<(f64, EdgeRow)> = stmt
        .query_map([], |row| {
            Ok(EdgeRow {
                signal_a: row.get(0)?,
                signal_b: row.get(1)?,
                total_seconds: row.get(2)?,
                last_seen: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .map(|e| {
            let age_days = (now_ms - e.last_seen) as f64 / 86_400_000.0;
            let w_eff =
                e.total_seconds * (-std::f64::consts::LN_2 / half_life_days * age_days).exp();
            (w_eff, e)
        })
        .collect();

    edges.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut parent: HashMap<String, String> = HashMap::new();

    fn find(parent: &mut HashMap<String, String>, x: &str) -> String {
        if !parent.contains_key(x) {
            parent.insert(x.to_string(), x.to_string());
            return x.to_string();
        }
        let p = parent[x].clone();
        if p == x {
            return x.to_string();
        }
        let root = find(parent, &p);
        parent.insert(x.to_string(), root.clone());
        root
    }

    fn union(parent: &mut HashMap<String, String>, a: &str, b: &str) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra == rb {
            return;
        }
        let ra_is_duet = ra.starts_with("duet:");
        let rb_is_duet = rb.starts_with("duet:");
        if ra_is_duet && !rb_is_duet {
            parent.insert(rb, ra);
        } else if rb_is_duet && !ra_is_duet {
            parent.insert(ra, rb);
        } else if ra < rb {
            parent.insert(rb, ra);
        } else {
            parent.insert(ra, rb);
        }
    }

    for (w_eff, edge) in &edges {
        if *w_eff < threshold_seconds {
            break;
        }
        union(&mut parent, &edge.signal_a, &edge.signal_b);
    }

    let signals: Vec<String> = parent.keys().cloned().collect();
    let mut result = HashMap::new();
    for signal in signals {
        let rep = find(&mut parent, &signal);
        result.insert(signal, rep);
    }
    Ok(result)
}

/// Returns the cluster representative for `signal`, or `signal` itself if not in any cluster.
pub fn cluster_at<'a>(clusters: &'a HashMap<String, String>, signal: &'a str) -> &'a str {
    clusters.get(signal).map(|s| s.as_str()).unwrap_or(signal)
}

// ---------------------------------------------------------------------------
// Dominant cluster
// ---------------------------------------------------------------------------

pub fn dominant_cluster(
    db: &Connection,
    clusters: &HashMap<String, String>,
    half_life_days: f64,
) -> Result<Option<String>, LocationError> {
    let now_ms = Utc::now().timestamp_millis();

    struct EdgeRow {
        signal_a: String,
        signal_b: String,
        total_seconds: f64,
        last_seen: i64,
    }

    let mut stmt = db
        .prepare("SELECT signal_a, signal_b, total_seconds, last_seen FROM co_occurrences")?;

    let edges: Vec<EdgeRow> = stmt
        .query_map([], |row| {
            Ok(EdgeRow {
                signal_a: row.get(0)?,
                signal_b: row.get(1)?,
                total_seconds: row.get(2)?,
                last_seen: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut cluster_weight: HashMap<String, f64> = HashMap::new();
    for edge in &edges {
        let age_days = (now_ms - edge.last_seen) as f64 / 86_400_000.0;
        let w_eff =
            edge.total_seconds * (-std::f64::consts::LN_2 / half_life_days * age_days).exp();
        let rep_a = cluster_at(clusters, &edge.signal_a).to_string();
        let rep_b = cluster_at(clusters, &edge.signal_b).to_string();
        *cluster_weight.entry(rep_a.clone()).or_insert(0.0) += w_eff;
        if rep_b != rep_a {
            *cluster_weight.entry(rep_b).or_insert(0.0) += w_eff;
        }
    }

    let mut dominant: Option<String> = None;
    let mut best_weight = 0.0f64;
    for (rep, weight) in &cluster_weight {
        let is_better = *weight > best_weight
            || (*weight == best_weight
                && dominant.as_deref().map(|d| rep.as_str() < d).unwrap_or(false));
        if is_better {
            dominant = Some(rep.clone());
            best_weight = *weight;
        }
    }
    Ok(dominant)
}

// ---------------------------------------------------------------------------
// Period collection
// ---------------------------------------------------------------------------

pub fn collect_duet_periods(duet_db_path: Option<&Utf8Path>) -> Vec<SignalPeriod> {
    let path = match duet_db_path {
        Some(p) => p,
        None => return vec![],
    };

    let conn = match Connection::open_with_flags(
        path.as_std_path(),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    struct AnchorRow {
        anchor_date: f64,
        anchor_location: String,
    }

    let mut stmt = match conn.prepare(
        "SELECT ao.anchorDate, ao.anchorLocation FROM anchorOccurrence ao
         JOIN locations l ON l.uuid = ao.anchorLocation
         WHERE l.type != ?1
         ORDER BY ao.anchorDate",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let anchors: Vec<AnchorRow> =
        match stmt.query_map(rusqlite::params![HOME_LOCATION_TYPE], |row| {
            Ok(AnchorRow { anchor_date: row.get(0)?, anchor_location: row.get(1)? })
        }) {
            Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
            Err(_) => return vec![],
        };

    let now_ms = Utc::now().timestamp_millis();
    let mut periods = Vec::with_capacity(anchors.len());
    for i in 0..anchors.len() {
        let start_ms = (anchors[i].anchor_date * 1000.0).round() as i64;
        let end_ms = if i + 1 < anchors.len() {
            (anchors[i + 1].anchor_date * 1000.0).round() as i64
        } else {
            now_ms
        };
        periods.push(SignalPeriod {
            start_ms,
            end_ms,
            signal: format!("duet:{}", anchors[i].anchor_location),
        });
    }
    periods
}

pub fn collect_subnet_periods(wifi_log_db: &Connection) -> Vec<SignalPeriod> {
    struct IpRow {
        timestamp: i64,
        subnet: String,
    }

    let mut stmt = match wifi_log_db
        .prepare("SELECT timestamp, subnet FROM ip_events ORDER BY timestamp")
    {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let rows: Vec<IpRow> = match stmt
        .query_map([], |row| {
            Ok(IpRow { timestamp: row.get(0)?, subnet: row.get(1)? })
        }) {
        Ok(mapped) => mapped.filter_map(|r| r.ok()).collect(),
        Err(_) => return vec![],
    };

    let now_ms = Utc::now().timestamp_millis();
    let mut periods = Vec::new();
    for i in 0..rows.len() {
        if rows[i].subnet.is_empty() {
            continue;
        }
        let start_ms = rows[i].timestamp;
        let end_ms = if i + 1 < rows.len() { rows[i + 1].timestamp } else { now_ms };
        periods.push(SignalPeriod {
            start_ms,
            end_ms,
            signal: format!("subnet:{}", rows[i].subnet),
        });
    }
    periods
}

// ---------------------------------------------------------------------------
// Clip duet periods to office subnet windows
// ---------------------------------------------------------------------------

fn clip_duet_to_office_subnets(
    duet_periods: &[SignalPeriod],
    office_subnet_periods: &[SignalPeriod],
) -> Vec<SignalPeriod> {
    let mut clipped = Vec::new();
    for dp in duet_periods {
        for sp in office_subnet_periods {
            if sp.start_ms >= dp.end_ms {
                break;
            }
            if sp.end_ms <= dp.start_ms {
                continue;
            }
            let start = dp.start_ms.max(sp.start_ms);
            let end = dp.end_ms.min(sp.end_ms);
            if end > start {
                clipped.push(SignalPeriod {
                    start_ms: start,
                    end_ms: end,
                    signal: dp.signal.clone(),
                });
            }
        }
    }
    clipped
}

// ---------------------------------------------------------------------------
// Load RTO data
// ---------------------------------------------------------------------------

pub fn load_rto_data(
    rto_db: &Connection,
    duet_db_path: Option<&Utf8Path>,
    wifi_log_db: &Connection,
    dates: &[&str],
    office_ssid: &str,
) -> Result<RtoData, LocationError> {
    let schema_version: i64 = rto_db
        .query_row(
            "SELECT value FROM schema_meta WHERE key = 'version'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if schema_version < RTO_SCHEMA_VERSION {
        rto_db.execute_batch("BEGIN")?;
        rto_db
            .execute(
                "UPDATE evidence SET location = 'duet:' || location
                 WHERE location IS NOT NULL AND location NOT LIKE '%:%'",
                [],
            )?;
        rto_db
            .execute(
                "INSERT INTO schema_meta (key, value) VALUES ('version', ?1)
                 ON CONFLICT (key) DO UPDATE SET value = excluded.value",
                rusqlite::params![RTO_SCHEMA_VERSION],
            )?;
        rto_db.execute_batch("COMMIT")?;
    }

    let mut ni_stmt = wifi_log_db
        .prepare("SELECT DISTINCT subnet FROM network_identities WHERE ssid = ?1")?;
    let office_subnets: std::collections::HashSet<String> = ni_stmt
        .query_map(rusqlite::params![office_ssid], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    let all_subnet_periods = collect_subnet_periods(wifi_log_db);
    let office_subnet_periods: Vec<SignalPeriod> = all_subnet_periods
        .iter()
        .filter(|p| {
            let subnet = p.signal.strip_prefix("subnet:").unwrap_or("");
            office_subnets.contains(subnet)
        })
        .cloned()
        .collect();

    let duet_periods = collect_duet_periods(duet_db_path);
    let clipped_duet_periods = clip_duet_to_office_subnets(&duet_periods, &office_subnet_periods);

    let since_ms = watermark(rto_db);
    let now_ms = Utc::now().timestamp_millis();
    update_co_occurrences(
        rto_db,
        &clipped_duet_periods,
        &office_subnet_periods,
        since_ms,
        MIN_OVERLAP_SECONDS,
    )?;
    set_watermark(rto_db, now_ms)?;

    let clusters = resolve_location_clusters(rto_db, HALF_LIFE_DAYS, CLUSTER_THRESHOLD_SECONDS)?;
    let dominant = dominant_cluster(rto_db, &clusters, HALF_LIFE_DAYS)?;

    let blocks: Vec<RtoBlock> = if dates.is_empty() {
        vec![]
    } else {
        let placeholders: String = (1..=dates.len())
            .map(|i| format!("?{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT first, last, location FROM evidence
             WHERE date IN ({placeholders}) AND location IS NOT NULL
             ORDER BY first"
        );
        let mut stmt = rto_db.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::ToSql> =
            dates.iter().map(|d| d as &dyn rusqlite::ToSql).collect();
        stmt.query_map(params.as_slice(), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?))
        })?
        .filter_map(|r| r.ok())
        .map(|(first, last, location)| RtoBlock {
            first_ms: first,
            last_ms: last,
            location: cluster_at(&clusters, &location).to_string(),
        })
        .collect()
    };

    let mut all_periods: Vec<RtoBlock> = clipped_duet_periods
        .iter()
        .map(|p| RtoBlock {
            first_ms: p.start_ms,
            last_ms: p.end_ms,
            location: cluster_at(&clusters, &p.signal).to_string(),
        })
        .chain(office_subnet_periods.iter().map(|p| RtoBlock {
            first_ms: p.start_ms,
            last_ms: p.end_ms,
            location: cluster_at(&clusters, &p.signal).to_string(),
        }))
        .collect();
    all_periods.sort_by_key(|b| b.first_ms);

    let mut seen_reps: std::collections::HashSet<String> = blocks
        .iter()
        .chain(all_periods.iter())
        .map(|b| b.location.clone())
        .filter(|loc| loc.starts_with("duet:"))
        .collect();
    if let Some(ref dom) = dominant {
        seen_reps.remove(dom);
    }
    let mut other_ids: Vec<String> = seen_reps.into_iter().collect();
    other_ids.sort();

    Ok(RtoData { blocks, all_periods, dominant_id: dominant, other_ids })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{
        cluster_at, dominant_cluster, resolve_location_clusters,
    };
    use crate::{CLUSTER_THRESHOLD_SECONDS, HALF_LIFE_DAYS};
    use chrono::Utc;
    use rusqlite::Connection;
    use std::collections::HashMap;

    fn make_in_memory_rto_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE co_occurrences (
                signal_a      TEXT    NOT NULL,
                signal_b      TEXT    NOT NULL,
                total_seconds REAL    NOT NULL DEFAULT 0,
                last_seen     INTEGER NOT NULL,
                PRIMARY KEY (signal_a, signal_b)
            );
            CREATE TABLE co_occurrence_watermark (
                id            INTEGER NOT NULL PRIMARY KEY CHECK (id = 1),
                processed_ms  INTEGER NOT NULL
            );",
        )
        .unwrap();
        conn
    }

    #[test]
    fn cluster_at_known_signal() {
        let mut clusters = HashMap::new();
        clusters.insert("subnet:10.0.0.0/24".to_string(), "duet:abc".to_string());
        assert_eq!(cluster_at(&clusters, "subnet:10.0.0.0/24"), "duet:abc");
    }

    #[test]
    fn cluster_at_unknown_signal() {
        let clusters: HashMap<String, String> = HashMap::new();
        assert_eq!(cluster_at(&clusters, "subnet:10.0.0.0/24"), "subnet:10.0.0.0/24");
    }

    #[test]
    fn resolve_location_clusters_merges_above_threshold() {
        let db = make_in_memory_rto_db();
        let now_ms = Utc::now().timestamp_millis();
        db.execute(
            "INSERT INTO co_occurrences (signal_a, signal_b, total_seconds, last_seen)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["duet:aaa", "subnet:10.0.0.0/24", 10000.0_f64, now_ms],
        )
        .unwrap();
        let clusters = resolve_location_clusters(&db, HALF_LIFE_DAYS, CLUSTER_THRESHOLD_SECONDS).unwrap();
        assert_eq!(clusters.get("duet:aaa").map(|s| s.as_str()), Some("duet:aaa"));
        assert_eq!(
            clusters.get("subnet:10.0.0.0/24").map(|s| s.as_str()),
            Some("duet:aaa")
        );
    }

    #[test]
    fn resolve_location_clusters_no_merge_below_threshold() {
        let db = make_in_memory_rto_db();
        let old_ms = 1_000_000i64;
        db.execute(
            "INSERT INTO co_occurrences (signal_a, signal_b, total_seconds, last_seen)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["duet:aaa", "subnet:10.0.0.0/24", 1.0_f64, old_ms],
        )
        .unwrap();
        let clusters = resolve_location_clusters(&db, HALF_LIFE_DAYS, CLUSTER_THRESHOLD_SECONDS).unwrap();
        let rep_duet = clusters.get("duet:aaa").map(|s| s.as_str()).unwrap_or("duet:aaa");
        let rep_subnet =
            clusters.get("subnet:10.0.0.0/24").map(|s| s.as_str()).unwrap_or("subnet:10.0.0.0/24");
        assert_ne!(rep_duet, rep_subnet);
    }

    #[test]
    fn dominant_cluster_basic() {
        let db = make_in_memory_rto_db();
        let now_ms = Utc::now().timestamp_millis();
        db.execute(
            "INSERT INTO co_occurrences (signal_a, signal_b, total_seconds, last_seen)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["duet:office-a", "subnet:10.1.0.0/24", 50000.0_f64, now_ms],
        )
        .unwrap();
        db.execute(
            "INSERT INTO co_occurrences (signal_a, signal_b, total_seconds, last_seen)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["duet:office-b", "subnet:10.2.0.0/24", 100.0_f64, now_ms],
        )
        .unwrap();
        let clusters = resolve_location_clusters(&db, HALF_LIFE_DAYS, CLUSTER_THRESHOLD_SECONDS).unwrap();
        let dominant = dominant_cluster(&db, &clusters, HALF_LIFE_DAYS).unwrap();
        assert_eq!(dominant.as_deref(), Some("duet:office-a"));
    }
}
