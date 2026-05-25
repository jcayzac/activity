//! Parses macOS wifi.log files and caches IP acquisition/loss events.
//!
//! Also maintains a `network_identities` table correlating (ssid, subnet) pairs
//! from cross-referencing with Biome WiFi session data, and a
//! `subnet_cooccurrence` table for union-find location grouping.

use anyhow::Context as _;
use camino::Utf8Path;
use chrono::Datelike as _;
use rusqlite::Connection;
use tokio::process::Command;

use crate::biome_wifi::WifiSession;

const WIFI_LOG_DIR: &str = "/private/var/log";
const LIVE_PARSE_TTL_MS: i64 = 5 * 60_000;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct IpEvent {
    pub time_ms: i64,
    pub ip: String,
    pub subnet: String,
}

pub use timeline::WifiEvent as WifiIpRow;

// ---------------------------------------------------------------------------
// Subnet computation
// ---------------------------------------------------------------------------

fn ip_to_u32(ip: &str) -> Option<u32> {
    let mut result: u32 = 0;
    for (i, octet) in ip.split('.').enumerate() {
        if i >= 4 {
            return None;
        }
        let o: u32 = octet.parse().ok()?;
        if o > 255 {
            return None;
        }
        result = (result << 8) | o;
    }
    Some(result)
}

fn mask_to_prefix_len(mask: &str) -> u32 {
    mask.split('.')
        .filter_map(|o| o.parse::<u32>().ok())
        .map(|o| o.count_ones())
        .sum()
}

pub fn compute_subnet(ip: &str, mask: &str) -> Option<String> {
    let ip_int = ip_to_u32(ip)?;
    let mask_int = ip_to_u32(mask)?;
    let network_int = ip_int & mask_int;
    let prefix = mask_to_prefix_len(mask);
    let octets = [
        (network_int >> 24) & 0xff,
        (network_int >> 16) & 0xff,
        (network_int >> 8) & 0xff,
        network_int & 0xff,
    ];
    Some(format!(
        "{}.{}.{}.{}/{}",
        octets[0], octets[1], octets[2], octets[3], prefix
    ))
}

// ---------------------------------------------------------------------------
// Cache schema
// ---------------------------------------------------------------------------

fn ensure_schema(db: &Connection) -> anyhow::Result<()> {
    db.execute_batch(
        "PRAGMA journal_mode=WAL;
         CREATE TABLE IF NOT EXISTS ip_events (
             timestamp INTEGER NOT NULL PRIMARY KEY,
             ip        TEXT    NOT NULL,
             subnet    TEXT    NOT NULL
         );
         CREATE TABLE IF NOT EXISTS network_identities (
             ssid         TEXT    NOT NULL,
             subnet       TEXT    NOT NULL,
             observations INTEGER NOT NULL DEFAULT 1,
             PRIMARY KEY (ssid, subnet)
         );
         CREATE TABLE IF NOT EXISTS subnet_cooccurrence (
             subnet_a TEXT NOT NULL,
             subnet_b TEXT NOT NULL,
             PRIMARY KEY (subnet_a, subnet_b)
         );
         CREATE TABLE IF NOT EXISTS parse_coverage (
             source    TEXT    NOT NULL PRIMARY KEY,
             parsed_at INTEGER NOT NULL
         );",
    )
    .context("failed to initialise wifi-log cache schema")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Coverage helpers
// ---------------------------------------------------------------------------

fn get_parsed_at(db: &Connection, source: &str) -> anyhow::Result<Option<i64>> {
    let mut stmt = db.prepare_cached("SELECT parsed_at FROM parse_coverage WHERE source = ?1")?;
    let mut rows = stmt.query([source])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

fn set_parsed_at(db: &Connection, source: &str, ts: i64) -> anyhow::Result<()> {
    db.execute(
        "INSERT INTO parse_coverage (source, parsed_at) VALUES (?1, ?2)
         ON CONFLICT (source) DO UPDATE SET parsed_at = excluded.parsed_at",
        rusqlite::params![source, ts],
    )?;
    Ok(())
}

fn cached_range(db: &Connection) -> anyhow::Result<(Option<i64>, Option<i64>)> {
    let mut stmt = db.prepare_cached("SELECT MIN(timestamp), MAX(timestamp) FROM ip_events")?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        let earliest: Option<i64> = row.get(0)?;
        let latest: Option<i64> = row.get(1)?;
        Ok((earliest, latest))
    } else {
        Ok((None, None))
    }
}

// ---------------------------------------------------------------------------
// Log line parsing
// ---------------------------------------------------------------------------

/// State machine for parsing consecutive wifi.log lines within one timestamp block.
struct LineParser {
    current_ts: Option<i64>,
    ip: Option<String>,
    mask: Option<String>,
    events: Vec<IpEvent>,
    year: i32,
}

impl LineParser {
    fn new(year: i32) -> Self {
        LineParser {
            current_ts: None,
            ip: None,
            mask: None,
            events: Vec::new(),
            year,
        }
    }

    fn flush(&mut self) {
        let Some(ts) = self.current_ts else { return };
        if let (Some(ip), Some(mask)) = (self.ip.take(), self.mask.take()) {
            if ip == "0.0.0.0" {
                self.events.push(IpEvent {
                    time_ms: ts,
                    ip: String::new(),
                    subnet: String::new(),
                });
            } else if let Some(subnet) = compute_subnet(&ip, &mask) {
                self.events.push(IpEvent {
                    time_ms: ts,
                    ip,
                    subnet,
                });
            }
        } else {
            self.ip = None;
            self.mask = None;
        }
    }

    fn feed_line(&mut self, line: &str) {
        // Format: "Tue May 19 11:45:33.009  ..."
        // Match: weekday(3) SP month(3) SP+ day SP time
        if let Some(ts) = parse_wifi_log_timestamp(line, self.year)
            && Some(ts) != self.current_ts
        {
            self.flush();
            self.current_ts = Some(ts);
            self.ip = None;
            self.mask = None;
        }

        // IP Address line (not Router/Gateway/Default)
        if !line.contains("Router")
            && !line.contains("Gateway")
            && !line.contains("Default")
            && let Some(ip) = extract_field(line, "IP Address: ")
        {
            self.ip = Some(ip.to_string());
        }
        if let Some(mask) = extract_field(line, "IP subnet mask: ") {
            self.mask = Some(mask.to_string());
        }
    }

    fn finish(mut self) -> Vec<IpEvent> {
        self.flush();
        self.events
    }
}

fn extract_field<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    // Find the field anywhere in the line
    let idx = line.find(prefix)?;
    let rest = &line[idx + prefix.len()..];
    // Take until whitespace or end
    let end = rest
        .find(|c: char| c.is_ascii_whitespace())
        .unwrap_or(rest.len());
    Some(&rest[..end])
}

/// Parses the timestamp from a wifi.log line: `"Tue May 19 11:45:33.009"`.
/// Returns milliseconds since Unix epoch using the provided year.
pub fn parse_wifi_log_timestamp(line: &str, year: i32) -> Option<i64> {
    // Expected prefix: "DDD MMM  D HH:MM:SS.mmm" or "DDD MMM DD HH:MM:SS.mmm"
    // We need at least 24 chars for the prefix.
    if line.len() < 20 {
        return None;
    }
    // Skip weekday (3 chars) and space
    if &line[3..4] != " " {
        return None;
    }
    let month_str = &line[4..7];
    let month = month_abbr_to_num(month_str)?;

    // Day: may be " 5" or "19"
    if &line[7..8] != " " {
        return None;
    }
    let day_str = line[8..10].trim();
    let day: u32 = day_str.parse().ok()?;

    if &line[10..11] != " " {
        return None;
    }
    // Time: HH:MM:SS.mmm
    let time_part = &line[11..];
    let h: u32 = time_part.get(0..2)?.parse().ok()?;
    if time_part.get(2..3)? != ":" {
        return None;
    }
    let mi: u32 = time_part.get(3..5)?.parse().ok()?;
    if time_part.get(5..6)? != ":" {
        return None;
    }
    let s: u32 = time_part.get(6..8)?.parse().ok()?;

    use chrono::TimeZone as _;
    let dt = chrono::Local
        .with_ymd_and_hms(year, month, day, h, mi, s)
        .single()?;
    Some(dt.timestamp_millis())
}

fn month_abbr_to_num(abbr: &str) -> Option<u32> {
    match abbr {
        "Jan" => Some(1),
        "Feb" => Some(2),
        "Mar" => Some(3),
        "Apr" => Some(4),
        "May" => Some(5),
        "Jun" => Some(6),
        "Jul" => Some(7),
        "Aug" => Some(8),
        "Sep" => Some(9),
        "Oct" => Some(10),
        "Nov" => Some(11),
        "Dec" => Some(12),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Parse + cache a single log file
// ---------------------------------------------------------------------------

async fn parse_and_cache_log_file(
    db: &Connection,
    path: &str,
    compressed: bool,
) -> anyhow::Result<usize> {
    // Determine year from file mtime
    let year = match tokio::fs::metadata(path).await {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(_) => chrono::Local::now().naive_local().year(),
        Ok(meta) => {
            let mtime = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            if mtime > 0 {
                chrono::DateTime::from_timestamp(mtime as i64, 0)
                    .map(|dt| dt.naive_utc().year())
                    .unwrap_or_else(|| chrono::Local::now().naive_local().year())
            } else {
                chrono::Local::now().naive_local().year()
            }
        }
    };

    let stdout_bytes = if compressed {
        let output = Command::new("bzcat")
            .arg(path)
            .output()
            .await
            .with_context(|| format!("failed to spawn bzcat for {path}"))?;
        output.stdout
    } else {
        match tokio::fs::read(path).await {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(e).with_context(|| format!("failed to read {path}")),
            Ok(b) => b,
        }
    };

    let content = String::from_utf8_lossy(&stdout_bytes);
    let mut parser = LineParser::new(year);
    for line in content.lines() {
        parser.feed_line(line);
    }
    let events = parser.finish();

    if events.is_empty() {
        return Ok(0);
    }

    let tx = db.unchecked_transaction()?;
    let mut inserted: usize = 0;
    {
        let mut ins = tx.prepare_cached(
            "INSERT OR IGNORE INTO ip_events (timestamp, ip, subnet) VALUES (?1, ?2, ?3)",
        )?;
        for e in &events {
            let changes = ins.execute(rusqlite::params![e.time_ms, e.ip, e.subnet])?;
            inserted += changes;
        }
    }
    tx.commit()?;
    Ok(inserted)
}

// ---------------------------------------------------------------------------
// Correlate with Biome sessions
// ---------------------------------------------------------------------------

fn correlate_with_biome(db: &Connection, biome_sessions: &[WifiSession]) -> anyhow::Result<()> {
    // Sort sessions by first_ms for binary search
    let mut sorted: Vec<&WifiSession> = biome_sessions.iter().collect();
    sorted.sort_by_key(|s| s.first_ms);
    let session_ends: Vec<i64> = sorted
        .iter()
        .map(|s| s.last_ms.max(s.first_ms + 60_000))
        .collect();

    let find_session = |ts: i64| -> Option<&WifiSession> {
        // Binary search for last session whose first_ms <= ts
        let mut lo = 0usize;
        let mut hi = sorted.len();
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if sorted[mid].first_ms <= ts {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        if lo == 0 {
            return None;
        }
        let idx = lo - 1;
        if ts <= session_ends[idx] {
            Some(sorted[idx])
        } else {
            None
        }
    };

    // Read all ip_events with non-empty subnet
    let ip_rows: Vec<(i64, String)> = {
        let mut stmt = db.prepare_cached(
            "SELECT timestamp, subnet FROM ip_events WHERE subnet != '' ORDER BY timestamp",
        )?;
        stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<_>>()?
    };

    // Build session→subnets map and time-window proximity list
    use std::collections::{HashMap, HashSet};
    let mut session_subnets: HashMap<(i64, i64), HashSet<String>> = HashMap::new();
    let mut session_ssid: HashMap<(i64, i64), String> = HashMap::new();

    let time_window_ms: i64 = 10 * 60_000;
    let mut ip_with_ssid: Vec<(i64, String, String)> = Vec::new(); // (ts, subnet, ssid)

    for (ts, subnet) in &ip_rows {
        if let Some(s) = find_session(*ts) {
            let key = (s.first_ms, s.last_ms);
            session_subnets
                .entry(key)
                .or_default()
                .insert(subnet.clone());
            session_ssid.entry(key).or_insert_with(|| s.ssid.clone());
            ip_with_ssid.push((*ts, subnet.clone(), s.ssid.clone()));
        }
    }

    let tx = db.unchecked_transaction()?;
    {
        let mut upsert_identity = tx.prepare_cached(
            "INSERT INTO network_identities (ssid, subnet, observations) VALUES (?1, ?2, 1)
             ON CONFLICT (ssid, subnet) DO UPDATE SET observations = observations + 1",
        )?;
        let mut upsert_cooc = tx.prepare_cached(
            "INSERT OR IGNORE INTO subnet_cooccurrence (subnet_a, subnet_b) VALUES (?1, ?2)",
        )?;

        for (key, subnets) in &session_subnets {
            let ssid = &session_ssid[key];
            let subnet_list: Vec<&String> = subnets.iter().collect();

            for subnet in &subnet_list {
                upsert_identity.execute(rusqlite::params![ssid, subnet])?;
            }

            // Co-occurrence pairs within session
            for i in 0..subnet_list.len() {
                for j in (i + 1)..subnet_list.len() {
                    let mut pair = [subnet_list[i].as_str(), subnet_list[j].as_str()];
                    pair.sort();
                    upsert_cooc.execute(rusqlite::params![pair[0], pair[1]])?;
                }
            }
        }

        // Time-window proximity co-occurrences under same SSID
        for i in 0..ip_with_ssid.len() {
            for j in (i + 1)..ip_with_ssid.len() {
                if ip_with_ssid[j].0 - ip_with_ssid[i].0 > time_window_ms {
                    break;
                }
                if ip_with_ssid[i].2 != ip_with_ssid[j].2 {
                    continue;
                }
                if ip_with_ssid[i].1 == ip_with_ssid[j].1 {
                    continue;
                }
                let mut pair = [ip_with_ssid[i].1.as_str(), ip_with_ssid[j].1.as_str()];
                pair.sort();
                upsert_cooc.execute(rusqlite::params![pair[0], pair[1]])?;
            }
        }
    }
    tx.commit()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Opens (or creates+populates) the wifi-log cache DB.
pub async fn open_wifi_log_db(
    cache_path: &Utf8Path,
    biome_sessions: &[WifiSession],
    needed_since_ms: i64,
) -> anyhow::Result<Connection> {
    if let Some(parent) = cache_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create cache dir {parent}"))?;
    }

    let db = Connection::open(cache_path.as_std_path())
        .with_context(|| format!("failed to open wifi-log cache at {cache_path}"))?;
    ensure_schema(&db)?;

    let now = chrono::Utc::now().timestamp_millis();
    let mut new_rows: usize = 0;

    // ---- Live log ----
    let live_path = format!("{WIFI_LOG_DIR}/wifi.log");
    let live_parsed_at = get_parsed_at(&db, "live")?;
    let needs_live = live_parsed_at
        .map(|t| now - t >= LIVE_PARSE_TTL_MS)
        .unwrap_or(true);

    if needs_live {
        match tokio::fs::metadata(&live_path).await {
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                anyhow::bail!(
                    "WiFi log cannot be accessed. Grant Full Disk Access to your terminal in System Settings."
                );
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(e).context("unexpected error accessing WiFi log");
            }
            Ok(_) => {
                new_rows += parse_and_cache_log_file(&db, &live_path, false).await?;
            }
        }
        set_parsed_at(&db, "live", now)?;
    }

    // ---- Archives ----
    let (earliest, _) = cached_range(&db)?;
    let needs_archives = earliest.map(|e| e > needed_since_ms).unwrap_or(true);

    if needs_archives {
        for i in 0..=20usize {
            let path = format!("{WIFI_LOG_DIR}/wifi.log.{i}.bz2");
            match tokio::fs::metadata(&path).await {
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    anyhow::bail!(
                        "WiFi log cannot be accessed. Grant Full Disk Access to your terminal in System Settings."
                    );
                }
                Err(_) => continue,
                Ok(_) => {}
            }
            let source = format!("archive:{i}");
            if get_parsed_at(&db, &source)?.is_some() {
                continue;
            }
            new_rows += parse_and_cache_log_file(&db, &path, true).await?;
            set_parsed_at(&db, &source, now)?;

            let (earliest2, _) = cached_range(&db)?;
            if let Some(e) = earliest2
                && e <= needed_since_ms
            {
                break;
            }
        }
    }

    if new_rows > 0 && !biome_sessions.is_empty() {
        correlate_with_biome(&db, biome_sessions)?;
    }

    Ok(db)
}

pub fn wifi_ip_events(db: &Connection) -> Vec<WifiIpRow> {
    let mut stmt = db
        .prepare_cached("SELECT timestamp, ip, subnet FROM ip_events ORDER BY timestamp")
        .expect("failed to prepare ip_events query");
    stmt.query_map([], |row| {
        Ok(WifiIpRow {
            time_ms: row.get(0)?,
            ip: row.get(1)?,
            subnet: row.get(2)?,
        })
    })
    .expect("failed to query ip_events")
    .filter_map(|r| r.ok())
    .collect()
}

pub fn build_location_groups(db: &Connection) -> std::collections::HashMap<String, String> {
    // Union-Find: canonical = lexicographically smallest in connected component
    let mut parent: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    fn find(parent: &mut std::collections::HashMap<String, String>, x: &str) -> String {
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

    fn union(parent: &mut std::collections::HashMap<String, String>, a: &str, b: &str) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra == rb {
            return;
        }
        if ra < rb {
            parent.insert(rb, ra);
        } else {
            parent.insert(ra, rb);
        }
    }

    // Load co-occurrences
    let pairs: Vec<(String, String)> = {
        let mut stmt = db
            .prepare_cached("SELECT subnet_a, subnet_b FROM subnet_cooccurrence")
            .expect("failed to prepare subnet_cooccurrence query");
        stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .expect("failed to query subnet_cooccurrence")
        .filter_map(|r| r.ok())
        .collect()
    };

    for (a, b) in &pairs {
        union(&mut parent, a, b);
    }

    // Resolve all known subnets
    let subnets: Vec<String> = {
        let mut stmt = db
            .prepare_cached("SELECT DISTINCT subnet FROM ip_events WHERE subnet != ''")
            .expect("failed to prepare subnet query");
        stmt.query_map([], |row| row.get::<_, String>(0))
            .expect("failed to query subnets")
            .filter_map(|r| r.ok())
            .collect()
    };

    let mut result = std::collections::HashMap::new();
    for subnet in &subnets {
        let root = find(&mut parent, subnet);
        result.insert(subnet.clone(), root);
    }
    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_subnet_basic() {
        // 192.168.1.10 / 255.255.255.0 → 192.168.1.0/24
        assert_eq!(
            compute_subnet("192.168.1.10", "255.255.255.0"),
            Some("192.168.1.0/24".to_string())
        );
    }

    #[test]
    fn compute_subnet_slash16() {
        // 10.0.5.1 / 255.255.0.0 → 10.0.0.0/16
        assert_eq!(
            compute_subnet("10.0.5.1", "255.255.0.0"),
            Some("10.0.0.0/16".to_string())
        );
    }

    #[test]
    fn compute_subnet_slash8() {
        assert_eq!(
            compute_subnet("172.16.0.1", "255.0.0.0"),
            Some("172.0.0.0/8".to_string())
        );
    }

    #[test]
    fn compute_subnet_invalid_ip() {
        assert!(compute_subnet("not.an.ip", "255.255.255.0").is_none());
    }

    #[test]
    fn parse_wifi_log_timestamp_basic() {
        // "Tue May 19 11:45:33.009" — space-padded day
        let line = "Tue May 19 11:45:33.009 something";
        let ts = parse_wifi_log_timestamp(line, 2024);
        assert!(ts.is_some(), "expected Some, got None for line: {line}");
    }

    #[test]
    fn parse_wifi_log_timestamp_single_digit_day() {
        // "Mon Jan  5 09:00:00.000"
        let line = "Mon Jan  5 09:00:00.000 something";
        let ts = parse_wifi_log_timestamp(line, 2024);
        assert!(ts.is_some(), "expected Some for single-digit day");
    }

    #[test]
    fn parse_wifi_log_timestamp_invalid() {
        assert!(parse_wifi_log_timestamp("not a timestamp", 2024).is_none());
    }
}
