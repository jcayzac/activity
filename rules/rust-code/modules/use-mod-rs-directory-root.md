# Use `mod.rs` for the root of a directory module

Rule id: `modules/use-mod-rs-directory-root`

**Rationale:** A reader should be able to treat `foo.rs` as self-contained and
`foo/` as the complete module tree. This keeps browsing, searching, renaming,
and review predictable.

**Origins:** `P-MOD` [EP].

---

<!-- References -->

[EP]: https://raw.githubusercontent.com/epage/epage.github.io/refs/heads/source/blog/dev/rust-style.md "Ed Page's Rust Style Guide"
