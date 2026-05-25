# Eagerly implement standard interoperability traits when they are semantically correct

Rule id: `traits/implement-standard-interoperability-traits`

**Rationale:** Downstream crates cannot add these impls later because of the
orphan rules. If you omit an obvious impl, you permanently make the API less
useful.

**Origins:** `C-COMMON-TRAITS`, `C-CONV-TRAITS`, `C-COLLECT`, `C-SERDE`,
`C-DEBUG`, `C-DEBUG-NONEMPTY` [API]; Item 10, "Familiarize yourself with
standard traits" [ER].

---

<!-- References -->

[API]: https://github.com/rust-lang/api-guidelines/blob/master/src/SUMMARY.md "The Rust API Guidelines"
[ER]: https://www.effective-rust.com/ "Effective Rust"
