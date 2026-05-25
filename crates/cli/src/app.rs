// CLI entry points: parse arguments, invoke use-case, render output.

use miette::miette;

use config::{cache_dir, load_config};
use periods::get_periods_for_dates;
use report::{build_day_report, build_month_report};
use timeline::date_utils::build_month_dates;
use timeline::today;

use crate::terminal::TerminalRenderer;

pub async fn run_day(date: &str, color: bool) -> miette::Result<()> {
    let today = today();
    let cache = cache_dir().map_err(miette::Report::from)?;
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

pub async fn run_month(yyyymm: &str, color: bool) -> miette::Result<()> {
    let today = today();
    let cache = cache_dir().map_err(miette::Report::from)?;
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
