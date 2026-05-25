# Name conversions, getters, iterators, and constructors the standard way

Rule id: `naming/standard-conversion-getter-iterator`

**Rationale:** These names communicate cost, ownership, mutability, and API
role. They let users predict behavior without reading the implementation.

**Origins:** `C-CONV`, `C-GETTER`, `C-ITER`, `C-ITER-TY`, `C-CTOR` [API];
`Names` [RSG]; Item 7, "Use builders for complex types" [ER].

---

<!-- References -->

[API]: https://github.com/rust-lang/api-guidelines/blob/master/src/SUMMARY.md "The Rust API Guidelines"
[RSG]: https://github.com/rust-lang/rust/blob/main/src/doc/style-guide/src/SUMMARY.md "The official Rust Style Guide"
[ER]: https://www.effective-rust.com/ "Effective Rust"
