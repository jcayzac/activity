# activity

A macOS CLI tool that reconstructs a timeline of computer usage for a given day or month. It reads from several macOS system data sources, classifies each period as active work, a break, or transit, and reports whether the day was worked from the office (RTO — Return To Office).

## Prerequisites

- macOS (tested on macOS 15+)
- Terminal must have **Full Disk Access** granted in System Settings → Privacy & Security for data sources that require it (Powerlog, Biome streams, knowledgeC.db, WiFi log). The tool will display a clear error message if access is missing.

## Installation

```sh
cargo install --path crates/cli
```

Or build without installing:

```sh
cargo build --release
# Binary is at target/release/activity
```

## Configuration

On first run, a template config file is created at:

```
$XDG_CONFIG_HOME/io.github.jcayzac.activity/config.toml
# or if XDG_CONFIG_HOME is not set:
~/.config/io.github.jcayzac.activity/config.toml
```

Edit it and set your office WiFi SSID:

```toml
[office]
ssid = "CorpWifi"
```

The tool will not run until `ssid` is configured.

## Usage

```
activity YYYY-MM-DD|YYYYMMDD     # day report
activity YYYY-MM|YYYYMM          # month report

Options:
  --color=never|always|auto      # default: auto
```

**Day report:** a table with Start, End, Duration, Type, and RTO columns. Active periods are highlighted; breaks and transit appear in a secondary color. The RTO column shows a check mark for the configured office and superscript symbols for other known office locations. Total active time appears at the bottom.

**Month report:** one row per active day showing first/last timestamps, total active time, RTO indicators, and break/transit markers.

## Cache

Processed data is cached in SQLite databases under:

```
$XDG_CACHE_HOME/io.github.jcayzac.activity/
# or if XDG_CACHE_HOME is not set:
~/.cache/io.github.jcayzac.activity/
```

Some macOS data sources have a rolling retention window of around 10 days. Data outside that window is preserved only if it was previously cached.
