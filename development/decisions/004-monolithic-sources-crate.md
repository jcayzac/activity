# ADR 004: sources is a single infrastructure adapter crate

## Status

Accepted (with known trade-offs)

## Context

Six macOS data-source adapters (Biome InFocus, Biome WiFi, powerlog, unified log, wifi.log, knowledgeC) are implemented as sub-modules of a single `crates/sources` crate. These adapters have different external dependencies, failure modes, and change drivers.

## Decision

Keep all six adapters in one `sources` crate for now. The `interval_cache` utility, which is a caching concern rather than a macOS data source, is moved to `crates/periods` where it belongs architecturally.

Splitting into per-adapter crates would provide cleaner build boundaries and independent versioning, but at the cost of six new crate manifests, more workspace boilerplate, and no immediate testability benefit (all adapters require Full Disk Access to test against real data).

## Consequences

- `sources` has a heterogeneous set of dependencies (`tokio`, `rusqlite`, `proto`, `futures`).
- Adding a new data source means adding a module to `sources`, which is low friction.
- If independent versioning or conditional compilation of adapters becomes necessary, splitting is the migration path.
- `interval_cache` is NOT in `sources`; it is in `periods` (ADR 002).
