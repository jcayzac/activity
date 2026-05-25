Ingestion and caching for all macOS data sources used by the `activity` tool. Each module reads from one system source and stores results in a local SQLite database so that subsequent runs do not re-parse raw system files.

The public modules and their primary exports are:

- `powerlog` — reads the macOS Powerlog database; exports `BlEvent` (backlight on/off), `FocusEvent` (app focus), `AggScreenOn` (aggregated screen-on buckets), and associated query functions.
- `unified_log` — queries `log show` output; exports `InputEvent`/`InputEventKind` for keyboard events and `ScreenEvent`/`ScreenEventKind` for lock/unlock events.
- `wifi_log` — parses `/private/var/log/wifi.log`; exports `IpEvent`, `WifiIpRow`, and `build_location_groups` which maps IP address changes to subnet periods.
- `biome_infocus` — reads Biome App.InFocus SEGB files via the `proto` crate; exports `InFocusEvent` and caching functions (`import_infocus_events`, `all_infocus_events`, `infocus_coverage`).
- `biome_wifi` — reads Biome WiFi connection stream files; exports `WifiSession` and `collect_biome_sessions`.
- `knowledge` — reads `knowledgeC.db` via SQLite; exports `FocusPeriod` and caching functions (`import_knowledge_focus_periods`, `all_focus_periods`, `knowledge_coverage`).
- `interval_cache` — generic interval cache layer; stores pre-built `Vec<Interval>` rows keyed by date so that the timeline does not need to be rebuilt on each run.
