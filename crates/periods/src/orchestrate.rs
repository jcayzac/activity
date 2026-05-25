use std::collections::HashMap;

use camino::Utf8PathBuf;

use location::{load_rto_data, open_rto_db};
use sources::{
    aggregate_screen_on, all_bl_events, all_focus_events, all_focus_periods, all_infocus_events,
    build_location_groups, collect_biome_sessions, import_infocus_events,
    import_knowledge_focus_periods, infocus_coverage, knowledge_coverage, open_infocus_db,
    open_knowledge_db, open_powerlog_db, open_unified_log_db, open_wifi_log_db, read_input_events,
    read_screen_events, wifi_ip_events,
};
use timeline::{
    AggScreenOnBucket, FrontmostEvent, InitialState, InputKind, Interval, IntervalLabel,
    PointFocusEvent, RawEvent, RawEventKind, ScreenEventKind, TimelineInputs,
    annotate_intervals_with_location, attribute_periods_to_date, build_aggregate_events,
    build_periods, build_timeline,
    date_utils::{next_day, six_am_of},
    effective_day_ms, merge_focus_streams,
};

use crate::interval_cache::{
    open_interval_cache_db, read_cached_intervals, should_reuse_cache, write_cached_intervals,
};
use crate::types::{PeriodsError, PeriodsResult};

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

/// Builds activity period intervals for the given `dates`.
///
/// All data is read from macOS system databases; results are cached in
/// `cache_dir`. `home_dir` is the user's home directory (`$HOME`).
/// `office_ssid` identifies the office network for RTO detection.
///
/// # Errors
///
/// Returns an error if any required data source cannot be opened or queried.
pub async fn get_periods_for_dates(
    dates: &[&str],
    today: &str,
    cache_dir: &Utf8PathBuf,
    home_dir: Option<&str>,
    office_ssid: &str,
) -> Result<PeriodsResult, PeriodsError> {
    let biome_wifi_db_path = cache_dir.join("biome-wifi.db");
    let wifi_log_db_path = cache_dir.join("wifi-log.db");
    let unified_log_db_path = cache_dir.join("unified-log.db");
    let powerlog_db_path = cache_dir.join("powerlog.db");
    let interval_cache_db_path = cache_dir.join("active-on.db");
    let knowledge_db_cache_path = cache_dir.join("focus-periods.db");
    let infocus_db_path = cache_dir.join("infocus.db");
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

    // Step 1: Open interval cache and check which dates need recompute.
    let cache_db = open_interval_cache_db(&interval_cache_db_path)?;

    let mut intervals: HashMap<String, Vec<Interval>> = HashMap::new();
    let mut uncached_dates: Vec<&str> = Vec::new();
    let mut provisional_fallbacks: HashMap<&str, Vec<Interval>> = HashMap::new();

    for &date in dates {
        let cached = read_cached_intervals(&cache_db, date)?;

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
    let location_groups = build_location_groups(&wifi_log_db)?;

    // Step 6: Open unified log DB.
    let unified_log_db = open_unified_log_db(&unified_log_db_path, all_dates_start_ms).await?;
    let screen_stream =
        read_screen_events(&unified_log_db, all_dates_start_ms, all_dates_end_ms)?;
    let input_stream = read_input_events(&unified_log_db, all_dates_start_ms, all_dates_end_ms)?;
    drop(unified_log_db);

    // Step 7: Open powerlog DB.
    let powerlog_db = open_powerlog_db(&powerlog_db_path, all_dates_start_ms).await?;

    // Step 8: Get BL stream.
    let bl_stream = all_bl_events(&powerlog_db)?;

    // Step 9: Open per-source focus cache DBs and import supplementary focus data.
    let knowledge_cache_db = open_knowledge_db(&knowledge_db_cache_path)?;
    let infocus_cache_db = open_infocus_db(&infocus_db_path)?;

    if let Some(ref kdb_path) = knowledge_db_path
        && kdb_path.exists()
    {
        let since = knowledge_coverage(&knowledge_cache_db)?;
        let _ = import_knowledge_focus_periods(&knowledge_cache_db, kdb_path, since);
    }
    if let Some(ref if_dir) = infocus_stream_dir
        && if_dir.exists()
    {
        let since = infocus_coverage(&infocus_cache_db)?;
        let _ = import_infocus_events(&infocus_cache_db, if_dir, since).await;
    }

    // Step 10: Collect focus periods and point events.
    let focus_periods: Vec<_> = all_focus_periods(&knowledge_cache_db)?
        .into_iter()
        .filter(|p| p.bundle_id != "com.apple.loginwindow")
        .collect();

    let powerlog_focus = all_focus_events(&powerlog_db)?;
    let infocus_events = all_infocus_events(&infocus_cache_db)?;

    let focus_point_events: Vec<PointFocusEvent> = powerlog_focus
        .into_iter()
        .chain(infocus_events)
        .filter(|e| e.bundle_id != "com.apple.loginwindow")
        .collect();

    // Step 11: Merge focus streams.
    let merged_focus = merge_focus_streams(&focus_periods, &focus_point_events);

    let frontmost_events: Vec<FrontmostEvent> = merged_focus
        .iter()
        .map(|e| FrontmostEvent { time_ms: e.time_ms, bundle_id: e.bundle_id.clone() })
        .collect();

    // Step 12: Get wifi IP events.
    let wifi_events = wifi_ip_events(&wifi_log_db)?;

    // Helper: get initial state for a window start.
    let get_initial_state = |window_start: i64| -> InitialState {
        let bl_on = last_before_by(&bl_stream, window_start, |e| e.time_ms)
            .map(|e| e.active)
            .unwrap_or(false);
        let screen_locked = last_before_by(&screen_stream, window_start, |e| e.time_ms)
            .map(|e| e.kind == ScreenEventKind::Lock)
            .unwrap_or(false);

        let ip_canon = {
            let maybe = last_before_by(&wifi_events, window_start, |e| e.time_ms);
            let idx = maybe.map(|m| {
                wifi_events.iter().rposition(|e| e.time_ms == m.time_ms).unwrap_or(0)
            });
            let found_subnet = if let Some(i) = idx {
                (0..=i).rev().find_map(|j| {
                    if !wifi_events[j].subnet.is_empty() && wifi_events[j].time_ms < window_start {
                        Some(wifi_events[j].subnet.clone())
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

        InitialState { bl_on, screen_locked, ip_canon, last_input_time }
    };

    // Step 13: Process uncached dates.
    if !uncached_dates.is_empty() {
        let now = chrono::Utc::now().timestamp_millis();
        let effective_today = effective_day_ms(now);

        let powerlog_dates: std::collections::HashSet<String> =
            bl_stream.iter().map(|e| effective_day_ms(e.time_ms)).collect();

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
                    wifi_events: &wifi_events,
                },
                &location_groups,
                window_start,
                clamped_end,
                initial,
            );

            if !date_intervals.iter().any(|iv| iv.label == IntervalLabel::Active)
                && !powerlog_dates.contains(date)
            {
                let agg = aggregate_screen_on(&powerlog_db, window_start, window_end)?;
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
                    .map(|e| RawEvent { time_ms: e.time_ms, kind: RawEventKind::Kbd })
                    .collect();

                let agg_events = build_aggregate_events(&agg_buckets, &soft_events);

                let bl_as_raw: Vec<RawEvent> = bl_stream
                    .iter()
                    .map(|e| RawEvent {
                        time_ms: e.time_ms,
                        kind: if e.active { RawEventKind::HardOpen } else { RawEventKind::HardClose },
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

            if !date_intervals.iter().any(|iv| iv.label == IntervalLabel::Active)
                && let Some(fallback) = provisional_fallbacks.get(date)
            {
                date_intervals = fallback.clone();
            }

            let has_open_period = date == effective_today
                && !date_intervals.is_empty()
                // `last_ms` is a stored wall-clock timestamp; `now` is also wall-clock.
                // The 5-second window intentionally uses wall-clock comparison.
                && date_intervals.last().map(|iv| iv.last_ms >= now - 5_000).unwrap_or(false);
            let is_provisional = date >= today || date == effective_today.as_str();

            if date <= today && (!date_intervals.is_empty() || date < today) {
                write_cached_intervals(
                    &cache_db,
                    date,
                    &date_intervals,
                    is_provisional || has_open_period,
                )?;
            }

            intervals.insert(date.to_string(), date_intervals);
        }
    }

    // Step 14: Load RTO data and annotate intervals.
    let rto_db = open_rto_db(&rto_db_path).map_err(anyhow::Error::from)?;
    let duet_path_ref = duet_db_path.as_deref();
    let rto_data = load_rto_data(&rto_db, duet_path_ref, &wifi_log_db, dates, office_ssid)
        .map_err(anyhow::Error::from)?;

    for ivs in intervals.values_mut() {
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
