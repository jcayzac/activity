// Use-case orchestration: data loading, timeline building, report rendering.

use std::collections::HashMap;

use camino::Utf8PathBuf;
use miette::miette;

use config::load_config;
use location::{load_rto_data, open_rto_db};
use report::{build_day_report, build_month_report};
use sources::{
    aggregate_screen_on, all_bl_events, all_focus_events, all_focus_periods, all_infocus_events,
    build_location_groups, collect_biome_sessions, import_infocus_events,
    import_knowledge_focus_periods, infocus_coverage, init_infocus_schema, init_knowledge_schema,
    interval_cache::{
        open_interval_cache_db, read_cached_intervals, should_reuse_cache, write_cached_intervals,
    },
    knowledge_coverage, open_focus_cache_db, open_powerlog_db, open_unified_log_db,
    open_wifi_log_db, read_input_events, read_screen_events, wifi_ip_events,
};
use timeline::{
    AggScreenOnBucket, FrontmostEvent, InitialState, InputKind, Interval, IntervalLabel,
    PointFocusEvent, RawEvent, RawEventKind, ScreenEventKind, TimelineInputs,
    annotate_intervals_with_location, attribute_periods_to_date, build_aggregate_events,
    build_periods, build_timeline,
    date_utils::{build_month_dates, format_local_date, next_day, six_am_of},
    effective_day_ms, merge_focus_streams,
};

use crate::terminal::TerminalRenderer;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn cache_dir() -> miette::Result<Utf8PathBuf> {
    let base = if let Ok(xdg) = std::env::var("XDG_CACHE_HOME")
        && !xdg.is_empty()
    {
        Utf8PathBuf::from(xdg)
    } else {
        let home = std::env::var("HOME").map_err(|_| miette!("$HOME is not set"))?;
        Utf8PathBuf::from(home).join(".cache")
    };
    Ok(base.join("io.github.jcayzac.activity"))
}

fn last_before_by<T>(arr: &[T], t: i64, time_fn: impl Fn(&T) -> i64) -> Option<&T> {
    if arr.is_empty() {
        return None;
    }
    let mut lo: usize = 0;
    let mut hi: usize = arr.len() - 1;
    let mut result: Option<usize> = None;
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        if time_fn(&arr[mid]) < t {
            result = Some(mid);
            if mid == arr.len() - 1 {
                break;
            }
            lo = mid + 1;
        } else {
            if mid == 0 {
                break;
            }
            hi = mid - 1;
        }
    }
    result.map(|i| &arr[i])
}

// ---------------------------------------------------------------------------
// PeriodsResult
// ---------------------------------------------------------------------------

pub struct PeriodsResult {
    pub intervals_by_date: HashMap<String, Vec<Interval>>,
    pub dominant_id: Option<String>,
    pub other_ids: Vec<String>,
}

// ---------------------------------------------------------------------------
// PeriodError
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum PeriodError {
    #[error("{0}")]
    Source(#[from] anyhow::Error),
}

// ---------------------------------------------------------------------------
// get_periods_for_dates
// ---------------------------------------------------------------------------

pub async fn get_periods_for_dates(
    dates: &[&str],
    today: &str,
    cache_dir: &Utf8PathBuf,
    home_dir: Option<&str>,
    office_ssid: &str,
) -> Result<PeriodsResult, PeriodError> {
    // Paths derived from cache_dir and home_dir
    let biome_wifi_db_path = cache_dir.join("biome-wifi.db");
    let wifi_log_db_path = cache_dir.join("wifi-log.db");
    let unified_log_db_path = cache_dir.join("unified-log.db");
    let powerlog_db_path = cache_dir.join("powerlog.db");
    let interval_cache_db_path = cache_dir.join("active-on.db");
    let focus_cache_db_path = cache_dir.join("focus-cache.db");
    let rto_db_path = cache_dir.join("rto.db");

    let biome_stream_dir: Option<Utf8PathBuf> = home_dir.map(|h| {
        Utf8PathBuf::from(h).join("Library/Biome/streams/restricted/_DKEvent.Wifi.Connection/local")
    });
    let infocus_stream_dir: Option<Utf8PathBuf> = home_dir
        .map(|h| Utf8PathBuf::from(h).join("Library/Biome/streams/restricted/App.InFocus/local"));
    let knowledge_db_path: Option<Utf8PathBuf> = home_dir
        .map(|h| Utf8PathBuf::from(h).join("Library/Application Support/Knowledge/knowledgeC.db"));
    let duet_db_path: Option<Utf8PathBuf> =
        home_dir.map(|h| Utf8PathBuf::from(h).join("Library/DuetExpertCenter/_ATXDataStore.db"));

    // Step 1: Open interval cache DB and check which dates need recompute.
    let cache_db = open_interval_cache_db(&interval_cache_db_path)?;

    let mut intervals: HashMap<String, Vec<Interval>> = HashMap::new();
    let mut uncached_dates: Vec<&str> = Vec::new();
    let mut provisional_fallbacks: HashMap<&str, Vec<Interval>> = HashMap::new();

    for &date in dates {
        let cached = read_cached_intervals(&cache_db, date);
        if let Some(ref c) = cached
            && !c.provisional
            && should_reuse_cache(date, today)
        {
            intervals.insert(date.to_string(), c.intervals.clone());
            continue;
        }
        if let Some(c) = cached
            && c.provisional
        {
            provisional_fallbacks.insert(date, c.intervals);
        }
        uncached_dates.push(date);
    }

    // Step 2: Collect Biome WiFi sessions.
    let biome_sessions = if let Some(ref stream_dir) = biome_stream_dir {
        collect_biome_sessions(&biome_wifi_db_path, stream_dir).await?
    } else {
        collect_biome_sessions(&biome_wifi_db_path, &Utf8PathBuf::from("/nonexistent"))
            .await
            .unwrap_or_default()
    };

    // Step 3: Compute time range for all dates.
    let mut sorted_dates: Vec<&str> = dates.to_vec();
    sorted_dates.sort();
    let all_dates_start_ms = six_am_of(sorted_dates[0]);
    let all_dates_end_ms = six_am_of(&next_day(sorted_dates[sorted_dates.len() - 1]));

    // Step 4: Open wifi log DB.
    let wifi_log_db =
        open_wifi_log_db(&wifi_log_db_path, &biome_sessions, all_dates_start_ms).await?;

    // Step 5: Build location groups.
    let location_groups = build_location_groups(&wifi_log_db);

    // Step 6: Open unified log DB and read screen/input streams.
    let unified_log_db = open_unified_log_db(&unified_log_db_path, all_dates_start_ms).await?;
    let screen_stream = read_screen_events(&unified_log_db, all_dates_start_ms, all_dates_end_ms);
    let input_stream = read_input_events(&unified_log_db, all_dates_start_ms, all_dates_end_ms);
    drop(unified_log_db);

    // Step 7: Open powerlog DB.
    let powerlog_db = open_powerlog_db(&powerlog_db_path, all_dates_start_ms).await?;

    // Step 8: Get BL stream.
    let bl_stream = all_bl_events(&powerlog_db);

    // Step 9: Open focus cache DB and import supplementary focus data.
    let focus_cache_db = open_focus_cache_db(&focus_cache_db_path)?;
    init_knowledge_schema(&focus_cache_db);
    init_infocus_schema(&focus_cache_db);

    if let Some(ref kdb_path) = knowledge_db_path
        && kdb_path.exists()
    {
        let since = knowledge_coverage(&focus_cache_db);
        let _ = import_knowledge_focus_periods(&focus_cache_db, kdb_path, since);
    }
    if let Some(ref if_dir) = infocus_stream_dir
        && if_dir.exists()
    {
        let since = infocus_coverage(&focus_cache_db);
        let _ = import_infocus_events(&focus_cache_db, if_dir, since).await;
    }

    // Step 10: Collect focus periods and point events.
    let focus_periods: Vec<_> = all_focus_periods(&focus_cache_db)
        .into_iter()
        .filter(|p| p.bundle_id != "com.apple.loginwindow")
        .collect();

    let focus_point_events: Vec<PointFocusEvent> = all_focus_events(&powerlog_db)
        .into_iter()
        .chain(all_infocus_events(&focus_cache_db).into_iter())
        .filter(|e| e.bundle_id != "com.apple.loginwindow")
        .collect();

    // Step 11: Merge focus streams.
    // merge_focus_streams returns Vec<PointFocusEvent>; build_timeline needs &[FrontmostEvent].
    let merged_focus = merge_focus_streams(&focus_periods, &focus_point_events);
    let frontmost_events: Vec<FrontmostEvent> = merged_focus
        .iter()
        .map(|e| FrontmostEvent {
            time_ms: e.time_ms,
            bundle_id: e.bundle_id.clone(),
        })
        .collect();

    // Step 12: Get wifi IP events.
    let wifi_ip_events = wifi_ip_events(&wifi_log_db);

    // Helper: get initial state for a window start
    let get_initial_state = |window_start: i64| -> InitialState {
        let bl_on = last_before_by(&bl_stream, window_start, |e| e.time_ms)
            .map(|e| e.active)
            .unwrap_or(false);
        let screen_locked = last_before_by(&screen_stream, window_start, |e| e.time_ms)
            .map(|e| e.kind == ScreenEventKind::Lock)
            .unwrap_or(false);

        // ip_canon: last ip_event with non-empty subnet before window_start
        let ip_canon = {
            // wifi_ip_events is sorted by time_ms
            let maybe = last_before_by(&wifi_ip_events, window_start, |e| e.time_ms);
            // search backwards for a non-empty subnet (like the SQL query does)
            let idx = maybe.map(|m| {
                wifi_ip_events
                    .iter()
                    .rposition(|e| e.time_ms == m.time_ms)
                    .unwrap_or(0)
            });
            let found_subnet = if let Some(i) = idx {
                // Find last non-empty subnet at or before i
                (0..=i).rev().find_map(|j| {
                    if !wifi_ip_events[j].subnet.is_empty()
                        && wifi_ip_events[j].time_ms < window_start
                    {
                        Some(wifi_ip_events[j].subnet.clone())
                    } else {
                        None
                    }
                })
            } else {
                None
            };
            found_subnet.map(|s| location_groups.get(&s).cloned().unwrap_or(s))
        };

        let input_row = last_before_by(&input_stream, window_start, |e| e.time_ms);
        let focus_row = last_before_by(&frontmost_events, window_start, |e| e.time_ms);
        let last_input_time = {
            let a = input_row.map(|e| e.time_ms).unwrap_or(0);
            let b = focus_row.map(|e| e.time_ms).unwrap_or(0);
            let m = a.max(b);
            if m == 0 { None } else { Some(m) }
        };

        InitialState {
            bl_on,
            screen_locked,
            ip_canon,
            last_input_time,
        }
    };

    // Step 13: Process uncached dates.
    if !uncached_dates.is_empty() {
        let now = chrono::Utc::now().timestamp_millis();
        let effective_today = effective_day_ms(now);

        // Build set of powerlog dates for aggregate fallback check
        let powerlog_dates: std::collections::HashSet<String> = bl_stream
            .iter()
            .map(|e| effective_day_ms(e.time_ms))
            .collect();

        for &date in &uncached_dates {
            let window_start = six_am_of(date);
            let window_end = six_am_of(&next_day(date));
            let clamped_end = window_end.min(now);

            let initial = get_initial_state(window_start);

            let mut date_intervals = build_timeline(
                TimelineInputs {
                    bl_events: &bl_stream,
                    screen_events: &screen_stream,
                    input_events: &input_stream,
                    frontmost_events: &frontmost_events,
                    wifi_events: &wifi_ip_events,
                },
                &location_groups,
                window_start,
                clamped_end,
                initial,
            );

            // Fallback to legacy buildPeriods for aggregate-only dates
            if !date_intervals
                .iter()
                .any(|iv| iv.label == IntervalLabel::Active)
                && !powerlog_dates.contains(date)
            {
                let agg = aggregate_screen_on(&powerlog_db, window_start, window_end);
                let agg_buckets: Vec<AggScreenOnBucket> = agg
                    .iter()
                    .map(|b| AggScreenOnBucket {
                        time_ms: b.time_ms,
                        screen_on_secs: b.screen_on_secs,
                    })
                    .collect();

                let soft_events: Vec<RawEvent> = input_stream
                    .iter()
                    .filter(|e| e.kind == InputKind::Kbd)
                    .map(|e| RawEvent {
                        time_ms: e.time_ms,
                        kind: RawEventKind::Kbd,
                    })
                    .collect();

                let agg_events = build_aggregate_events(&agg_buckets, &soft_events);

                let bl_as_raw: Vec<RawEvent> = bl_stream
                    .iter()
                    .map(|e| RawEvent {
                        time_ms: e.time_ms,
                        kind: if e.active {
                            RawEventKind::HardOpen
                        } else {
                            RawEventKind::HardClose
                        },
                    })
                    .collect();

                let screen_as_raw: Vec<RawEvent> = screen_stream
                    .iter()
                    .map(|e| RawEvent {
                        time_ms: e.time_ms,
                        kind: if e.kind == ScreenEventKind::Lock {
                            RawEventKind::HardClose
                        } else {
                            RawEventKind::HardOpen
                        },
                    })
                    .collect();

                let mut legacy_all: Vec<RawEvent> = bl_as_raw;
                legacy_all.extend(agg_events);
                legacy_all.extend(screen_as_raw);
                legacy_all.extend(soft_events.clone());

                let legacy_periods = build_periods(&mut legacy_all, now);
                let final_periods = attribute_periods_to_date(&legacy_periods, date, &soft_events);

                if !final_periods.is_empty() {
                    date_intervals = Vec::new();
                    for (i, p) in final_periods.iter().enumerate() {
                        date_intervals.push(Interval {
                            first_ms: p.first_ms,
                            last_ms: p.last_ms,
                            label: IntervalLabel::Active,
                            location: None,
                        });
                        if i + 1 < final_periods.len() {
                            date_intervals.push(Interval {
                                first_ms: p.last_ms,
                                last_ms: final_periods[i + 1].first_ms,
                                label: IntervalLabel::Break,
                                location: None,
                            });
                        }
                    }
                }
            }

            // Provisional fallback if still no active intervals
            if !date_intervals
                .iter()
                .any(|iv| iv.label == IntervalLabel::Active)
                && let Some(fallback) = provisional_fallbacks.get(date)
            {
                date_intervals = fallback.clone();
            }

            let has_open_period = date == effective_today
                && !date_intervals.is_empty()
                && date_intervals
                    .last()
                    .map(|iv| iv.last_ms >= now - 5_000)
                    .unwrap_or(false);
            let is_provisional = date >= today || date == effective_today.as_str();

            if date <= today && (!date_intervals.is_empty() || date < today) {
                write_cached_intervals(
                    &cache_db,
                    date,
                    &date_intervals,
                    is_provisional || has_open_period,
                );
            }

            intervals.insert(date.to_string(), date_intervals);
        }
    }

    // Step 14: Load RTO data and annotate intervals.
    let rto_db = open_rto_db(&rto_db_path)?;
    let duet_path_ref = duet_db_path.as_deref();
    let rto_data = load_rto_data(&rto_db, duet_path_ref, &wifi_log_db, dates, office_ssid);

    for (date, ivs) in intervals.iter_mut() {
        let _ = date; // used as key
        let annotated =
            annotate_intervals_with_location(ivs.clone(), &rto_data.blocks, &rto_data.all_periods);
        *ivs = annotated;
    }

    Ok(PeriodsResult {
        intervals_by_date: intervals,
        dominant_id: rto_data.dominant_id,
        other_ids: rto_data.other_ids,
    })
}

// ---------------------------------------------------------------------------
// run_day
// ---------------------------------------------------------------------------

pub async fn run_day(date: &str, color: bool) -> miette::Result<()> {
    let today = format_local_date(chrono::Utc::now().timestamp_millis());
    let cache = cache_dir()?;
    tokio::fs::create_dir_all(cache.as_std_path())
        .await
        .map_err(|e| miette!("failed to create cache dir: {e}"))?;

    let home = std::env::var("HOME").ok();
    let home_ref = home.as_deref();
    let config = load_config()?;

    let result = get_periods_for_dates(&[date], &today, &cache, home_ref, &config.office_ssid)
        .await
        .map_err(|e| miette!("{e:#}"))?;

    let intervals = result
        .intervals_by_date
        .get(date)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let report = build_day_report(
        date,
        intervals,
        result.dominant_id.as_deref(),
        &result.other_ids,
    );
    let renderer = TerminalRenderer { color };
    println!("{}", renderer.render_day(&report));
    Ok(())
}

// ---------------------------------------------------------------------------
// run_month
// ---------------------------------------------------------------------------

pub async fn run_month(yyyymm: &str, color: bool) -> miette::Result<()> {
    let today = format_local_date(chrono::Utc::now().timestamp_millis());
    let cache = cache_dir()?;
    tokio::fs::create_dir_all(cache.as_std_path())
        .await
        .map_err(|e| miette!("failed to create cache dir: {e}"))?;

    let home = std::env::var("HOME").ok();
    let home_ref = home.as_deref();
    let config = load_config()?;

    let dates = build_month_dates(yyyymm);
    let dates_ref: Vec<&str> = dates.iter().map(|s| s.as_str()).collect();

    let result = get_periods_for_dates(&dates_ref, &today, &cache, home_ref, &config.office_ssid)
        .await
        .map_err(|e| miette!("{e:#}"))?;

    let report = build_month_report(
        yyyymm,
        &dates,
        &result.intervals_by_date,
        result.dominant_id.as_deref(),
        &result.other_ids,
        &today,
    );
    let renderer = TerminalRenderer { color };
    println!("{}", renderer.render_month(&report));
    Ok(())
}
