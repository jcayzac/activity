# Separate private imports from public re-exports

Rule id: `imports/separate-imports-reexports`

**Rationale:** Private imports are implementation detail. Public re-exports are
part of the module's outward shape and belong near the visible API.

**Origins:** `M-PRIV-PUB-USE` [EP].

---

<!-- References -->

[EP]: https://raw.githubusercontent.com/epage/epage.github.io/refs/heads/source/blog/dev/rust-style.md "Ed Page's Rust Style Guide"
