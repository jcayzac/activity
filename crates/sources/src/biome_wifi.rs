//! Reads macOS Biome WiFi connection stream files and caches sessions.
//!
//! WiFi Biome files use marker-scan (not SEGB framing). Parsed sessions are
//! cached in a SQLite DB so past records survive Biome's rolling stream window.

use anyhow::Context as _;
use camino::Utf8Path;
use rusqlite::Connection;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct WifiSession {
    pub first_ms: i64,
    pub last_ms: i64,
    pub ssid: String,
}

// ---------------------------------------------------------------------------
// Cache schema
// ---------------------------------------------------------------------------

fn ensure_schema(db: &Connection) -> anyhow::Result<()> {
    db.execute_batch(
        "PRAGMA journal_mode=WAL;
         CREATE TABLE IF NOT EXISTS sessions (
             first INTEGER NOT NULL,
             last  INTEGER NOT NULL,
             ssid  TEXT    NOT NULL,
             PRIMARY KEY (first, last, ssid)
         );",
    )
    .context("failed to initialise biome-wifi schema")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Reads Biome WiFi stream files, merges with cache, returns all sessions.
///
/// Files with numeric names in `stream_dir` are processed in sorted order.
pub async fn collect_biome_sessions(
    cache_path: &Utf8Path,
    stream_dir: &Utf8Path,
) -> anyhow::Result<Vec<WifiSession>> {
    if let Some(parent) = cache_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create cache dir {parent}"))?;
    }

    let db = Connection::open(cache_path.as_std_path())
        .with_context(|| format!("failed to open biome-wifi cache at {cache_path}"))?;
    ensure_schema(&db)?;

    // Collect stream files (numeric names only, sorted)
    let mut stream_files: Vec<String> = Vec::new();
    match tokio::fs::read_dir(stream_dir.as_std_path()).await {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {}
        Err(e) => return Err(e).with_context(|| format!("failed to read stream dir {stream_dir}")),
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

    // Parse sessions from each file
    let mut new_sessions: Vec<WifiSession> = Vec::new();
    for path in &stream_files {
        let bytes = match tokio::fs::read(path).await {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e).with_context(|| format!("failed to read stream file {path}")),
        };
        for ev in proto::scan_wifi_records(&bytes) {
            new_sessions.push(WifiSession {
                first_ms: ev.first_ms,
                last_ms: ev.last_ms,
                ssid: ev.ssid,
            });
        }
    }

    // Insert new sessions
    if !new_sessions.is_empty() {
        let tx = db.unchecked_transaction()?;
        {
            let mut ins = tx.prepare_cached(
                "INSERT OR IGNORE INTO sessions (first, last, ssid) VALUES (?1, ?2, ?3)",
            )?;
            for s in &new_sessions {
                ins.execute(rusqlite::params![s.first_ms, s.last_ms, s.ssid])?;
            }
        }
        tx.commit()?;
    }

    // Read all sessions from cache
    let mut stmt =
        db.prepare_cached("SELECT first, last, ssid FROM sessions ORDER BY first, last")?;
    let sessions = stmt
        .query_map([], |row| {
            Ok(WifiSession {
                first_ms: row.get(0)?,
                last_ms: row.get(1)?,
                ssid: row.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("failed to read sessions from cache")?;

    Ok(sessions)
}
