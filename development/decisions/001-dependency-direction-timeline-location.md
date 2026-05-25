# ADR 001: timeline is the domain crate; location depends on timeline

## Status

Accepted

## Context

The initial implementation had `timeline` depending on `location` to use the `RtoBlock` type. `timeline` is the domain/policy crate (interval building, state machine, period logic). `location` is an infrastructure crate (SQLite I/O, Duet DB, Kruskal clustering). The dependency arrow was inverted: detail depended on stable policy, but policy also depended on detail.

This violated the dependency-direction rule: source code dependencies must point from volatile details toward stable policy.

## Decision

Move `RtoBlock` and `RtoData` into `timeline`. `location` now depends on `timeline` for those types and re-exports them for its own callers.

Dependency graph after this change:

```
cli / periods  →  location  →  timeline
cli / periods  →  sources   →  timeline
cli / periods  →  report    →  timeline
```

All arrows point toward `timeline` (the stable policy core).

## Consequences

- `timeline` has no dependency on `location` or any infrastructure crate.
- `location` re-exports `RtoBlock` and `RtoData` via `pub use timeline::...` for backward compatibility with callers.
- The domain model (`Interval`, `RtoBlock`, event types) lives entirely in `timeline` and is reachable without the SQLite dependency tree.
