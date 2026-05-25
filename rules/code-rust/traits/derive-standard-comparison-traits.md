# Derive standard comparison traits when structural semantics are correct

Rule id: `traits/derive-standard-comparison-traits`

**Rationale:** Manual comparison impls are easy to get subtly wrong. Derive is
shorter, safer, and easier to review.

**Origins:** `LANG-CMP-INV`, `LANG-CMP-DEFAULTS`, `LANG-CMP-DERIVE` [ANSSI];
`C-COMMON-TRAITS` [API]; Item 10, "Familiarize yourself with standard traits"
[ER].

---

<!-- References -->

[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
[API]: https://github.com/rust-lang/api-guidelines/blob/master/src/SUMMARY.md "The Rust API Guidelines"
[ER]: https://www.effective-rust.com/ "Effective Rust"
