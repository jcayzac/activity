# Development

## Toolchain

Rust 1.94.0 (2024 edition), pinned via `rust-toolchain.toml`. Use `rustup` to manage it.

## Building

```sh
cargo build              # debug
cargo build --release    # release
```

## Testing

```sh
cargo nextest run                   # all tests
cargo nextest run -p timeline       # tests for one crate
```

## Linting, formatting, and auditing

```sh
cargo lint               # clippy -D warnings
cargo lint-fix           # clippy --fix
cargo fmt-fix            # rustfmt
cargo fmt --all --check  # format check only
cargo audit -D warnings  # check dependencies for known vulnerabilities (fails on any advisory)
```

The `fmt-fix`, `lint`, and `lint-fix` aliases are defined in `.cargo/config.toml`. `cargo audit` is an external subcommand that must be installed once with `cargo install cargo-audit --locked`.

## Git hooks

`.githooks/pre-commit` runs `lint-fix`, `fmt-fix`, re-stages auto-fixes, then checks formatting. `.githooks/pre-push` runs the full test suite. The hooks path is set via `git config core.hooksPath .githooks` and is already configured in the repository.

## Contents

- [architecture/](architecture/) — crate structure, data flow, dependency graph
- [investigations/](investigations/) — research notes and analysis
- [adr/](adr/) — architecture decision records

## Rule books

Coding and architecture rules are in [`rules/`](../rules/):

- [`rules/code-rust/rules.md`](../rules/code-rust/rules.md) — Rust coding conventions
- [`rules/architecture-application/rules.md`](../rules/architecture-application/rules.md) — architecture rules
