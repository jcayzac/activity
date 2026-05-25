# Prefer iterator transforms for pure data transformations

Rule id: `function-structure/iterator-transforms-pure-data`

**Rationale:** Pure iterator pipelines make the dataflow explicit and reduce the
amount of indexing, temporary state, and loop bookkeeping the reader has to
carry.

**Origins:** Item 9, "Consider using iterator transforms instead of explicit
loops" [ER]; `F-COMBINATOR` [EP]. These sources align when the code is a pure
data transformation rather than effectful business logic.

---

<!-- References -->

[ER]: https://www.effective-rust.com/ "Effective Rust"
[EP]: https://raw.githubusercontent.com/epage/epage.github.io/refs/heads/source/blog/dev/rust-style.md "Ed Page's Rust Style Guide"
