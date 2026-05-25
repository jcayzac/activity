# Keep business logic in visible control-flow blocks

Rule id: `function-structure/visible-business-control-flow`

**Rationale:** Readers should be able to see the important branches without
mentally flattening a chain of early exits.

**Origins:** `F-VISUAL` [EP]; `Expressions` [RSG].

---

<!-- References -->

[EP]: https://raw.githubusercontent.com/epage/epage.github.io/refs/heads/source/blog/dev/rust-style.md "Ed Page's Rust Style Guide"
[RSG]: https://github.com/rust-lang/rust/blob/main/src/doc/style-guide/src/SUMMARY.md "The official Rust Style Guide"
