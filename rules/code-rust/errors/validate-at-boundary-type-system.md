# Validate at the boundary, preferably in the type system

Rule id: `errors/validate-at-boundary-type-system`

**Rationale:** Static validation catches errors earlier and once. Dynamic
validation duplicates work and pushes failures farther from the source.

**Origins:** `C-VALIDATE` [API]; Item 1, "Use the type system to express your
data structures" [ER]; Item 6, "Embrace the newtype pattern" [ER].

---

<!-- References -->

[API]: https://github.com/rust-lang/api-guidelines/blob/master/src/SUMMARY.md "The Rust API Guidelines"
[ER]: https://www.effective-rust.com/ "Effective Rust"
