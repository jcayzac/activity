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

## Linting and formatting

```sh
cargo lint               # clippy -D warnings
cargo lint-fix           # clippy --fix
cargo fmt-fix            # rustfmt
cargo fmt --all --check  # format check only
```

These aliases are defined in `.cargo/config.toml`.

## Git hooks

`.githooks/pre-commit` runs `lint-fix`, `fmt-fix`, re-stages auto-fixes, then checks formatting. `.githooks/pre-push` runs the full test suite. The hooks path is set via `git config core.hooksPath .githooks` and is already configured in the repository.

## Contents

- [architecture/](architecture/) — crate structure, data flow, dependency graph
- [investigations/](investigations/) — research notes and analysis
- [adr/](adr/) — architecture decision records

## Rule books

Coding and architecture rules are in [`rules/`](../rules/):

- [`rules/rust-code/rules.md`](../rules/rust-code/rules.md) — Rust coding conventions
- [`rules/application-architecture/rules.md`](../rules/application-architecture/rules.md) — architecture rules
