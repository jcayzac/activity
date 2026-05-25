# ADR 003: tokio is the async runtime

## Status

Accepted

## Context

Several data sources require spawning subprocesses (`log show`, `gunzip`, `bzcat`) and performing concurrent I/O (parallel archive decompression, reading multiple Biome stream files). An async runtime is needed.

## Decision

Use `tokio` as the async runtime. It is the dominant Rust async runtime with broad ecosystem support, stable APIs, and first-class support for subprocess management (`tokio::process::Command`) and concurrent task spawning (`tokio::spawn`).

Runtime-owned types (`tokio::process::Command`, `tokio::task::JoinHandle`) are confined to adapter crates (`sources`) and the entry point (`cli`). The domain (`timeline`) and use-case (`periods`) layers are runtime-agnostic: they accept owned `Vec<_>` results, not futures or streams.

## Consequences

- `timeline` and `location` have no `tokio` dependency.
- `periods` depends on `tokio` only to `.await` adapter futures in the orchestration function.
- If the runtime were replaced, only `cli` and `sources` would need changes.
