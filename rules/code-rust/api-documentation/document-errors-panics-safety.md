# Document `Errors`, `Panics`, and `Safety`

Rule id: `api-documentation/document-errors-panics-safety`

**Rationale:** Reviewers need to be able to compare implementation behavior
against a written contract. Undocumented failure and safety behavior makes APIs
impossible to reason about.

**Origins:** `C-FAILURE` [API]; Item 27, "Document public interfaces" [ER].

---

<!-- References -->

[API]: https://github.com/rust-lang/api-guidelines/blob/master/src/SUMMARY.md "The Rust API Guidelines"
[ER]: https://www.effective-rust.com/ "Effective Rust"
