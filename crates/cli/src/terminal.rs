// Terminal renderer: colored, padded text output.
// Port of lib/renderers/terminal.ts.
// Gruvbox-inspired palette (scheme 9).

use report::{DayReport, MonthReport};
use timeline::IntervalLabel;

// ---------------------------------------------------------------------------
// RTO symbols
// ---------------------------------------------------------------------------

const RTO_DOMINANT: &str = "\u{2713}\u{FE0E}"; // ✓︎ U+2713 U+FE0E
const RTO_SUPERSCRIPTS: &[&str] = &["²", "³", "⁴", "⁵", "⁶", "⁷", "⁸", "⁹"];
const RTO_ASTERISK: &str = "*";

fn rto_other_symbol(other_id: &str, other_ids: &[String]) -> String {
    if other_ids.len() == 1 {
        return RTO_ASTERISK.to_string();
    }
    let i = other_ids.iter().position(|id| id == other_id).unwrap_or(0);
    RTO_SUPERSCRIPTS
        .get(i)
        .map(|s| s.to_string())
        .unwrap_or_else(|| (i + 2).to_string())
}

// ---------------------------------------------------------------------------
// ANSI color helpers
// ---------------------------------------------------------------------------

fn ansi_bold_rgb24(s: &str, r: u8, g: u8, b: u8) -> String {
    format!("\x1b[1m\x1b[38;2;{r};{g};{b}m{s}\x1b[0m")
}

fn ansi_rgb24(s: &str, r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{r};{g};{b}m{s}\x1b[0m")
}

fn clr_date(s: &str, color: bool) -> String {
    if color {
        ansi_bold_rgb24(s, 0x83, 0xA5, 0x98)
    } else {
        s.to_string()
    }
}

fn clr_work(s: &str, color: bool) -> String {
    if color {
        ansi_bold_rgb24(s, 0xB8, 0xBB, 0x26)
    } else {
        s.to_string()
    }
}

fn clr_time(s: &str, color: bool) -> String {
    if color {
        ansi_rgb24(s, 0xEB, 0xDB, 0xB2)
    } else {
        s.to_string()
    }
}

fn clr_bk_dur(s: &str, color: bool) -> String {
    if color {
        ansi_rgb24(s, 0xFE, 0x80, 0x19)
    } else {
        s.to_string()
    }
}

fn clr_chrome(s: &str, color: bool) -> String {
    if color {
        ansi_rgb24(s, 0x7C, 0x6F, 0x64)
    } else {
        s.to_string()
    }
}

// ---------------------------------------------------------------------------
// Time/duration formatting
// ---------------------------------------------------------------------------

/// Returns HH:MM where HH can exceed 24 for next-day times.
/// `round_up = true` uses ceiling division of ms->minutes.
pub fn format_extended_time(base_date: &str, ts_ms: i64, round_up: bool) -> String {
    use chrono::{Datelike as _, Local, NaiveDate, TimeZone as _};

    let parts: Vec<u32> = base_date
        .split('-')
        .map(|p| p.parse().unwrap_or(0))
        .collect();
    let (year, month, day) = (parts[0] as i32, parts[1], parts[2]);
    let midnight = NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|d| {
            Local
                .with_ymd_and_hms(d.year(), d.month(), d.day(), 0, 0, 0)
                .single()
        })
        .map(|dt| dt.timestamp_millis())
        .unwrap_or(0);

    let ms = ts_ms - midnight;
    let total_minutes = if round_up {
        // ceiling division
        (ms + 59_999) / 60_000
    } else {
        ms / 60_000
    };
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{:02}:{:02}", hours, minutes)
}

/// Returns HHhMM format (e.g. "08h46").
pub fn format_duration(ms: i64) -> String {
    let total_minutes = ((ms as f64) / 60_000.0).round() as i64;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{:02}h{:02}", hours, minutes)
}

// ---------------------------------------------------------------------------
// Renderer
// ---------------------------------------------------------------------------

pub struct TerminalRenderer {
    pub color: bool,
}

impl TerminalRenderer {
    fn rto_symbol(
        &self,
        location_id: Option<&str>,
        dominant_id: Option<&str>,
        other_ids: &[String],
    ) -> String {
        let loc = match location_id {
            Some(l) => l,
            None => return String::new(),
        };
        if dominant_id == Some(loc) {
            return clr_work(RTO_DOMINANT, self.color);
        }
        clr_bk_dur(&rto_other_symbol(loc, other_ids), self.color)
    }

    fn rto_symbols(
        &self,
        location_ids: &[String],
        dominant_id: Option<&str>,
        other_ids: &[String],
    ) -> String {
        location_ids
            .iter()
            .map(|id| self.rto_symbol(Some(id), dominant_id, other_ids))
            .collect::<Vec<_>>()
            .join("")
    }

    fn rto_legend(
        &self,
        dominant_id: Option<&str>,
        other_ids: &[String],
        used_ids: &std::collections::HashSet<String>,
    ) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        if let Some(dom) = dominant_id
            && used_ids.contains(dom)
        {
            lines.push(
                clr_work(RTO_DOMINANT, self.color) + &clr_chrome("  Main office", self.color),
            );
        }
        let used_other_ids: Vec<&String> = other_ids
            .iter()
            .filter(|id| used_ids.contains(*id))
            .collect();
        if !used_other_ids.is_empty() {
            let symbols = used_other_ids
                .iter()
                .map(|id| clr_bk_dur(&rto_other_symbol(id, other_ids), self.color))
                .collect::<Vec<_>>()
                .join(" ");
            lines.push(symbols + &clr_chrome("  Other office", self.color));
        }
        lines
    }

    fn weekday_prefix(date: &str) -> String {
        use chrono::{Datelike as _, NaiveDate, Weekday};
        let parts: Vec<u32> = date.split('-').map(|p| p.parse().unwrap_or(0)).collect();
        let d = NaiveDate::from_ymd_opt(parts[0] as i32, parts[1], parts[2])
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        match d.weekday() {
            Weekday::Mon => "Mon",
            Weekday::Tue => "Tue",
            Weekday::Wed => "Wed",
            Weekday::Thu => "Thu",
            Weekday::Fri => "Fri",
            Weekday::Sat => "Sat",
            Weekday::Sun => "Sun",
        }
        .to_string()
    }

    pub fn render_day(&self, report: &DayReport) -> String {
        let DayReport {
            date,
            intervals,
            total_active_ms,
            dominant_id,
            other_ids,
        } = report;
        let date_label = format!("{} {}", Self::weekday_prefix(date), date);

        let has_active = intervals.iter().any(|iv| iv.label == IntervalLabel::Active);
        if !has_active {
            return format!(
                "{}\n\nNo user-initiated activity found.",
                clr_work(&date_label, self.color)
            );
        }

        struct Row {
            start: String,
            end: String,
            duration: String,
            kind: String, // "active", "break", "transit"
            location: Option<String>,
        }

        let rows: Vec<Row> = intervals
            .iter()
            .filter(|iv| iv.last_ms > iv.first_ms)
            .map(|iv| {
                let kind = match iv.label {
                    IntervalLabel::Active => "active",
                    IntervalLabel::Break => "break",
                    IntervalLabel::Transit => "transit",
                };
                // Transit intervals don't get a location in the display
                let location = if iv.label != IntervalLabel::Transit {
                    iv.location.clone()
                } else {
                    None
                };
                Row {
                    start: format_extended_time(date, iv.first_ms, false),
                    end: format_extended_time(date, iv.last_ms, false),
                    duration: format_duration(iv.last_ms - iv.first_ms),
                    kind: kind.to_string(),
                    location,
                }
            })
            .collect();

        let used_ids: std::collections::HashSet<String> =
            rows.iter().filter_map(|r| r.location.clone()).collect();
        let has_rto = !used_ids.is_empty();
        let rto_col = "RTO".len(); // = 3

        let start_width = rows
            .iter()
            .map(|r| r.start.len())
            .max()
            .unwrap_or(0)
            .max("Start".len());
        let end_width = rows
            .iter()
            .map(|r| r.end.len())
            .max()
            .unwrap_or(0)
            .max("End".len());
        let duration_width = rows
            .iter()
            .map(|r| r.duration.len())
            .max()
            .unwrap_or(0)
            .max("Duration".len());
        let type_width = rows
            .iter()
            .map(|r| r.kind.len())
            .max()
            .unwrap_or(0)
            .max("Type".len());
        let sep = "   "; // 3 spaces

        let rto_header = if has_rto {
            format!(
                "{}{}",
                sep,
                "RTO".to_string() + &" ".repeat(rto_col - "RTO".len())
            )
        } else {
            String::new()
        };

        // Build header text (unstyled) for measuring divider width
        let header_text = format!(
            "{}{}{}{}{}{}{}{}{}",
            pad_end("Start", start_width),
            sep,
            pad_start("End", end_width),
            sep,
            pad_start("Duration", duration_width),
            sep,
            pad_end("Type", type_width),
            rto_header,
            ""
        );
        let header = clr_chrome(&header_text, self.color);
        // Divider: repeat "─" to header_text.len() (byte count of unstyled text)
        let divider = clr_chrome(&"─".repeat(header_text.len()), self.color);

        let total_line = format!(
            "{}{}{}{}{}",
            " ".repeat(start_width),
            sep,
            clr_chrome(&pad_start("Total", end_width), self.color),
            sep,
            clr_work(
                &pad_start(&format_duration(*total_active_ms), duration_width),
                self.color
            ),
        );

        let dom_ref = dominant_id.as_deref();
        let legend = self.rto_legend(dom_ref, other_ids, &used_ids);

        let mut lines: Vec<String> = vec![
            clr_work(&date_label, self.color),
            String::new(),
            header,
            divider.clone(),
        ];

        for r in &rows {
            let start = clr_time(&pad_end(&r.start, start_width), self.color);
            let end = clr_time(&pad_start(&r.end, end_width), self.color);
            let dur = if r.kind == "active" {
                clr_work(&pad_start(&r.duration, duration_width), self.color)
            } else {
                clr_bk_dur(&pad_start(&r.duration, duration_width), self.color)
            };
            let kind = clr_chrome(&pad_end(&r.kind, type_width), self.color);
            let rto_cell = if has_rto {
                let sym = self.rto_symbol(r.location.as_deref(), dom_ref, other_ids);
                if !sym.is_empty() {
                    // sym is one symbol (display width 1) + spaces to fill rto_col
                    format!("{}{}{}", sep, sym, " ".repeat(rto_col - 1))
                } else {
                    format!("{}{}", sep, " ".repeat(rto_col))
                }
            } else {
                String::new()
            };
            lines.push(format!(
                "{}{}{}{}{}{}{}{}",
                start, sep, end, sep, dur, sep, kind, rto_cell
            ));
        }

        lines.push(divider);
        lines.push(total_line);

        if !legend.is_empty() {
            lines.push(String::new());
            lines.extend(legend);
        }

        lines.join("\n")
    }

    pub fn render_month(&self, report: &MonthReport) -> String {
        let MonthReport {
            yyyymm,
            rows,
            dominant_id,
            other_ids,
        } = report;

        let year: u32 = yyyymm[..4].parse().unwrap_or(0);
        let month: u32 = yyyymm[4..6].parse().unwrap_or(0);
        let month_name = match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown",
        };
        let title = format!("{} {}", month_name, year);

        if rows.is_empty() {
            return format!("{}\n\nNo activity found.", clr_work(&title, self.color));
        }

        struct TableRow {
            date_str: String,
            start: String,
            end: String,
            total: String,
            rto_str: String,
            breaks_str: String,
            locations: Vec<String>,
        }

        let dom_ref = dominant_id.as_deref();

        let table_rows: Vec<TableRow> = rows
            .iter()
            .map(|r| {
                let breaks_str = r
                    .breaks
                    .iter()
                    .map(|b| {
                        let tag = if b.label == IntervalLabel::Transit {
                            clr_chrome(" transit", self.color)
                        } else {
                            String::new()
                        };
                        format!(
                            "{}{}{}{}{}{}{}",
                            clr_time(
                                &format_extended_time(&r.date, b.first_ms, false),
                                self.color
                            ),
                            clr_chrome("\u{2192}", self.color), // →
                            clr_time(&format_extended_time(&r.date, b.last_ms, false), self.color),
                            " ",
                            clr_chrome("(", self.color),
                            clr_bk_dur(&format_duration(b.last_ms - b.first_ms), self.color),
                            tag + &clr_chrome(")", self.color),
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(&clr_chrome(", ", self.color));

                TableRow {
                    date_str: format!("{} {}:", Self::weekday_prefix(&r.date), r.date),
                    start: format_extended_time(&r.date, r.first_ms, false),
                    end: format_extended_time(&r.date, r.last_ms, false),
                    total: format_duration(r.total_active_ms),
                    rto_str: self.rto_symbols(&r.locations, dom_ref, other_ids),
                    breaks_str,
                    locations: r.locations.clone(),
                }
            })
            .collect();

        let has_rto = table_rows.iter().any(|r| !r.rto_str.is_empty());
        let max_symbols = table_rows
            .iter()
            .map(|r| r.locations.len())
            .max()
            .unwrap_or(0);
        let rto_col = "RTO".len().max(max_symbols); // max(3, max_symbols)

        let date_width = table_rows
            .iter()
            .map(|r| r.date_str.len())
            .max()
            .unwrap_or(0)
            .max("Date".len());
        let start_width = table_rows
            .iter()
            .map(|r| r.start.len())
            .max()
            .unwrap_or(0)
            .max("Start".len());
        let end_width = table_rows
            .iter()
            .map(|r| r.end.len())
            .max()
            .unwrap_or(0)
            .max("End".len());
        let total_width = table_rows
            .iter()
            .map(|r| r.total.len())
            .max()
            .unwrap_or(0)
            .max("Total".len());
        let sep = "  "; // 2 spaces

        let rto_header = if has_rto {
            format!("{}{}", sep, pad_end("RTO", rto_col))
        } else {
            String::new()
        };

        let header_text = format!(
            "{}{}{}{}{}{}{}{}{}{}{}",
            pad_end("Date", date_width),
            sep,
            pad_start("Start", start_width),
            sep,
            pad_start("End", end_width),
            sep,
            pad_start("Total", total_width),
            rto_header,
            sep,
            "Breaks",
            ""
        );
        let header = clr_chrome(&header_text, self.color);
        let divider = clr_chrome(&"─".repeat(header_text.len()), self.color);

        let used_ids: std::collections::HashSet<String> = table_rows
            .iter()
            .flat_map(|r| r.locations.clone())
            .collect();
        let legend = self.rto_legend(dom_ref, other_ids, &used_ids);

        let data_rows: Vec<String> = table_rows
            .iter()
            .map(|r| {
                let rto_cell = if has_rto {
                    if !r.rto_str.is_empty() {
                        // Each symbol has display width 1, but may be multi-byte.
                        // Padding: rto_col - r.locations.len() spaces.
                        format!(
                            "{}{}{}",
                            sep,
                            r.rto_str,
                            " ".repeat(rto_col - r.locations.len())
                        )
                    } else {
                        format!("{}{}", sep, " ".repeat(rto_col))
                    }
                } else {
                    String::new()
                };
                let breaks_part = if !r.breaks_str.is_empty() {
                    format!("{}{}", sep, r.breaks_str)
                } else {
                    String::new()
                };
                format!(
                    "{}{}{}{}{}{}{}{}{}",
                    clr_date(&pad_end(&r.date_str, date_width), self.color),
                    sep,
                    clr_time(&pad_start(&r.start, start_width), self.color),
                    sep,
                    clr_time(&pad_start(&r.end, end_width), self.color),
                    sep,
                    clr_work(&pad_start(&r.total, total_width), self.color),
                    rto_cell,
                    breaks_part,
                )
            })
            .collect();

        let mut lines: Vec<String> =
            vec![clr_work(&title, self.color), String::new(), header, divider];
        lines.extend(data_rows);

        if !legend.is_empty() {
            lines.push(String::new());
            lines.extend(legend);
        }

        lines.join("\n")
    }
}

// ---------------------------------------------------------------------------
// Padding helpers (ASCII strings — pad by byte count = char count for these)
// ---------------------------------------------------------------------------

fn pad_end(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s.len()))
    }
}

fn pad_start(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{}{}", " ".repeat(width - s.len()), s)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // format_duration
    #[test]
    fn format_duration_zero() {
        assert_eq!(format_duration(0), "00h00");
    }

    #[test]
    fn format_duration_30min() {
        assert_eq!(format_duration(30 * 60 * 1000), "00h30");
    }

    #[test]
    fn format_duration_90min() {
        assert_eq!(format_duration(90 * 60 * 1000), "01h30");
    }

    // format_extended_time
    // These tests use local time (chrono::Local), so we test relative offsets
    // by constructing the expected midnight from a known date and computing offsets.
    #[test]
    fn format_extended_time_midnight() {
        use chrono::{Datelike as _, Local, NaiveDate, TimeZone as _};
        let date = "2026-05-19";
        let d = NaiveDate::from_ymd_opt(2026, 5, 19).unwrap();
        let midnight = Local
            .with_ymd_and_hms(d.year(), d.month(), d.day(), 0, 0, 0)
            .single()
            .unwrap()
            .timestamp_millis();
        assert_eq!(format_extended_time(date, midnight, false), "00:00");
    }

    #[test]
    fn format_extended_time_6am() {
        use chrono::{Datelike as _, Local, NaiveDate, TimeZone as _};
        let date = "2026-05-19";
        let d = NaiveDate::from_ymd_opt(2026, 5, 19).unwrap();
        let midnight = Local
            .with_ymd_and_hms(d.year(), d.month(), d.day(), 0, 0, 0)
            .single()
            .unwrap()
            .timestamp_millis();
        let six_am = midnight + 6 * 60 * 60 * 1000;
        assert_eq!(format_extended_time(date, six_am, false), "06:00");
    }

    #[test]
    fn format_extended_time_next_day() {
        // 25h30 = midnight + 25.5h
        use chrono::{Datelike as _, Local, NaiveDate, TimeZone as _};
        let date = "2026-05-19";
        let d = NaiveDate::from_ymd_opt(2026, 5, 19).unwrap();
        let midnight = Local
            .with_ymd_and_hms(d.year(), d.month(), d.day(), 0, 0, 0)
            .single()
            .unwrap()
            .timestamp_millis();
        let ts = midnight + (25 * 60 + 30) * 60 * 1000;
        assert_eq!(format_extended_time(date, ts, false), "25:30");
    }

    // rto_other_symbol
    #[test]
    fn rto_other_symbol_single_other_is_asterisk() {
        let other_ids = vec!["office-b".to_string()];
        assert_eq!(rto_other_symbol("office-b", &other_ids), "*");
    }

    #[test]
    fn rto_other_symbol_multiple_superscript() {
        let other_ids = vec!["office-b".to_string(), "office-c".to_string()];
        // first one → index 0 → "²"
        assert_eq!(rto_other_symbol("office-b", &other_ids), "²");
        // second → index 1 → "³"
        assert_eq!(rto_other_symbol("office-c", &other_ids), "³");
    }

    #[test]
    fn rto_other_symbol_overflow_uses_number() {
        // Build 9 "other" offices so the 9th overflows RTO_SUPERSCRIPTS (len=8)
        let other_ids: Vec<String> = (0..9).map(|i| format!("office-{i}")).collect();
        // Index 8 → superscripts has index 0..7, so index 8 overflows → "10" (8+2)
        assert_eq!(rto_other_symbol("office-8", &other_ids), "10");
    }
}
