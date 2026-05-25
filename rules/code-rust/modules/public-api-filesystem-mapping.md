# Keep the public API mappable to the filesystem

Rule id: `modules/public-api-filesystem-mapping`

**Rationale:** A dependent should be able to navigate from an API path to the
code with minimal guessing. Hidden layout indirection makes review and debugging
slower.

**Origins:** `P-PATH-MOD`, `P-API` [EP]; `Modules` [RSG].

---

<!-- References -->

[EP]: https://raw.githubusercontent.com/epage/epage.github.io/refs/heads/source/blog/dev/rust-style.md "Ed Page's Rust Style Guide"
[RSG]: https://github.com/rust-lang/rust/blob/main/src/doc/style-guide/src/SUMMARY.md "The official Rust Style Guide"
