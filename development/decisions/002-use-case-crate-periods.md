# ADR 002: Use-case orchestration lives in the periods crate, not cli

## Status

Accepted

## Context

The 14-step `get_periods_for_dates` function, along with `PeriodsResult`, `PeriodError`, and related helpers, was implemented directly in `crates/cli/src/app.rs`. `cli` is an edge crate: it should parse arguments, invoke a use case, and render output. It must not own business decisions or application orchestration.

Having the use case in `cli` made it:
- Untestable in isolation (no way to call it without the CLI binary)
- Coupled to the presentation layer
- Invisible to any future alternative entry point (TUI, API server, etc.)

## Decision

Extract the use-case into a dedicated `crates/periods` crate. `cli` depends on `periods` and becomes a thin wrapper:

```
main.rs → parse args
app.rs  → call periods::get_periods_for_dates, render result
```

`periods` owns:
- `get_periods_for_dates` (the 14-step orchestration)
- `PeriodsResult`, `PeriodsError`
- The `interval_cache` module (caching layer between use-case and adapters)

## Consequences

- `cli` no longer imports `sources`, `location`, `rusqlite`, or `thiserror` directly.
- The use case is testable without the CLI.
- A future alternative entry point (daemon, API server) can depend on `periods` directly.
