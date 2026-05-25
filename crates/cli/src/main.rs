#![warn(clippy::all)]
#![forbid(unsafe_code)]

mod app;
mod terminal;

use miette::miette;

// ---------------------------------------------------------------------------
// Argument patterns (mirroring TypeScript exactly)
// ---------------------------------------------------------------------------
//
// DATE_PATTERN_DASHES:   ^(20\d{2})-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])$
// DATE_PATTERN_COMPACT:  ^(20\d{2})(0[1-9]|1[0-2])(0[1-9]|[12]\d|3[01])$
// MONTH_PATTERN_DASHES:  ^(20\d{2})-(0[1-9]|1[0-2])$
// MONTH_PATTERN_COMPACT: ^(20\d{2})(0[1-9]|1[0-2])$

enum ParsedArg {
    Day { date: String },
    Month { yyyymm: String },
}

fn parse_arg(arg: &str) -> miette::Result<ParsedArg> {
    // DATE_PATTERN_DASHES: ^(20\d{2})-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])$
    if let Some(caps) = match_date_dashes(arg) {
        return Ok(ParsedArg::Day { date: caps });
    }
    // DATE_PATTERN_COMPACT: ^(20\d{2})(0[1-9]|1[0-2])(0[1-9]|[12]\d|3[01])$
    if let Some(normalized) = match_date_compact(arg) {
        return Ok(ParsedArg::Day { date: normalized });
    }
    // MONTH_PATTERN_DASHES: ^(20\d{2})-(0[1-9]|1[0-2])$
    if let Some(yyyymm) = match_month_dashes(arg) {
        return Ok(ParsedArg::Month { yyyymm });
    }
    // MONTH_PATTERN_COMPACT: ^(20\d{2})(0[1-9]|1[0-2])$
    if let Some(yyyymm) = match_month_compact(arg) {
        return Ok(ParsedArg::Month { yyyymm });
    }
    Err(miette!(
        "unrecognized argument '{arg}'.\n\nUsage: activity YYYY-MM-DD|YYYYMMDD|YYYY-MM|YYYYMM [--color=never|always|auto]"
    ))
}

// Manual pattern matching without a regex crate dependency.

fn match_date_dashes(s: &str) -> Option<String> {
    // ^(20\d{2})-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])$
    let bytes = s.as_bytes();
    if bytes.len() != 10 {
        return None;
    }
    if bytes[0] != b'2' || bytes[1] != b'0' {
        return None;
    }
    if !bytes[2].is_ascii_digit() || !bytes[3].is_ascii_digit() {
        return None;
    }
    if bytes[4] != b'-' {
        return None;
    }
    if !is_valid_month(&s[5..7]) {
        return None;
    }
    if bytes[7] != b'-' {
        return None;
    }
    if !is_valid_day(&s[8..10]) {
        return None;
    }
    Some(s.to_string())
}

fn match_date_compact(s: &str) -> Option<String> {
    // ^(20\d{2})(0[1-9]|1[0-2])(0[1-9]|[12]\d|3[01])$
    let bytes = s.as_bytes();
    if bytes.len() != 8 {
        return None;
    }
    if bytes[0] != b'2' || bytes[1] != b'0' {
        return None;
    }
    if !bytes[2].is_ascii_digit() || !bytes[3].is_ascii_digit() {
        return None;
    }
    if !is_valid_month(&s[4..6]) {
        return None;
    }
    if !is_valid_day(&s[6..8]) {
        return None;
    }
    let year = &s[0..4];
    let month = &s[4..6];
    let day = &s[6..8];
    Some(format!("{year}-{month}-{day}"))
}

fn match_month_dashes(s: &str) -> Option<String> {
    // ^(20\d{2})-(0[1-9]|1[0-2])$
    let bytes = s.as_bytes();
    if bytes.len() != 7 {
        return None;
    }
    if bytes[0] != b'2' || bytes[1] != b'0' {
        return None;
    }
    if !bytes[2].is_ascii_digit() || !bytes[3].is_ascii_digit() {
        return None;
    }
    if bytes[4] != b'-' {
        return None;
    }
    if !is_valid_month(&s[5..7]) {
        return None;
    }
    let year = &s[0..4];
    let month = &s[5..7];
    Some(format!("{year}{month}"))
}

fn match_month_compact(s: &str) -> Option<String> {
    // ^(20\d{2})(0[1-9]|1[0-2])$
    let bytes = s.as_bytes();
    if bytes.len() != 6 {
        return None;
    }
    if bytes[0] != b'2' || bytes[1] != b'0' {
        return None;
    }
    if !bytes[2].is_ascii_digit() || !bytes[3].is_ascii_digit() {
        return None;
    }
    if !is_valid_month(&s[4..6]) {
        return None;
    }
    Some(s.to_string())
}

fn is_valid_month(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() != 2 {
        return false;
    }
    // 0[1-9]|1[0-2]
    match bytes[0] {
        b'0' => bytes[1] >= b'1' && bytes[1] <= b'9',
        b'1' => bytes[1] >= b'0' && bytes[1] <= b'2',
        _ => false,
    }
}

fn is_valid_day(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() != 2 {
        return false;
    }
    // 0[1-9]|[12]\d|3[01]
    match bytes[0] {
        b'0' => bytes[1] >= b'1' && bytes[1] <= b'9',
        b'1' | b'2' => bytes[1].is_ascii_digit(),
        b'3' => bytes[1] == b'0' || bytes[1] == b'1',
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Color detection
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
enum ColorChoice {
    Never,
    Always,
    Auto,
}

impl ColorChoice {
    fn resolve(self) -> bool {
        match self {
            ColorChoice::Never => false,
            ColorChoice::Always => true,
            ColorChoice::Auto => {
                // Respect NO_COLOR convention
                if std::env::var("NO_COLOR").is_ok() {
                    return false;
                }
                // Use IsTerminal (stable since Rust 1.70)
                use std::io::IsTerminal as _;
                std::io::stdout().is_terminal()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> miette::Result<()> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();

    // Parse --color flag
    let mut color_choice = ColorChoice::Auto;
    let mut remaining: Vec<String> = Vec::new();
    for a in args.drain(..) {
        match a.as_str() {
            "--color=never" => {
                color_choice = ColorChoice::Never;
            }
            "--color=always" => {
                color_choice = ColorChoice::Always;
            }
            "--color=auto" | "--color" => {
                color_choice = ColorChoice::Auto;
            }
            _ => {
                remaining.push(a);
            }
        }
    }
    args = remaining;

    if args.is_empty() {
        println!("Usage: activity YYYY-MM-DD|YYYYMMDD|YYYY-MM|YYYYMM [--color=never|always|auto]");
        println!();
        println!("Examples:");
        println!("  activity 2026-04-25");
        println!("  activity 20260425");
        println!("  activity 2026-04");
        println!("  activity 202604");
        return Ok(());
    }

    if args.len() != 1 {
        return Err(miette!("expected exactly one positional argument"));
    }

    let color = color_choice.resolve();
    let parsed = parse_arg(&args[0])?;

    match parsed {
        ParsedArg::Day { date } => app::run_day(&date, color).await,
        ParsedArg::Month { yyyymm } => app::run_month(&yyyymm, color).await,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{ParsedArg, parse_arg};

    #[test]
    fn parse_valid_date_dashes() {
        let result = parse_arg("2026-04-25");
        assert!(result.is_ok());
        match result.unwrap() {
            ParsedArg::Day { date } => assert_eq!(date, "2026-04-25"),
            _ => panic!("expected Day"),
        }
    }

    #[test]
    fn parse_valid_date_compact() {
        let result = parse_arg("20260425");
        assert!(result.is_ok());
        match result.unwrap() {
            ParsedArg::Day { date } => assert_eq!(date, "2026-04-25"),
            _ => panic!("expected Day"),
        }
    }

    #[test]
    fn parse_valid_month_dashes() {
        let result = parse_arg("2026-04");
        assert!(result.is_ok());
        match result.unwrap() {
            ParsedArg::Month { yyyymm } => assert_eq!(yyyymm, "202604"),
            _ => panic!("expected Month"),
        }
    }

    #[test]
    fn parse_valid_month_compact() {
        let result = parse_arg("202604");
        assert!(result.is_ok());
        match result.unwrap() {
            ParsedArg::Month { yyyymm } => assert_eq!(yyyymm, "202604"),
            _ => panic!("expected Month"),
        }
    }

    #[test]
    fn parse_invalid_returns_error() {
        assert!(parse_arg("foobar").is_err());
        assert!(parse_arg("2026-13").is_err());
        assert!(parse_arg("2026-00").is_err());
        assert!(parse_arg("2026-04-00").is_err());
        assert!(parse_arg("2026-04-32").is_err());
        assert!(parse_arg("19990101").is_err()); // year doesn't start with 20
    }
}
