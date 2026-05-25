# Prefer safe Rust, and forbid `unsafe` when you do not need it

Rule id: `unsafe/prefer-safe-rust-forbid-unneeded-unsafe`

**Rationale:** Safe Rust is the default soundness boundary. The less `unsafe`
exists, the less proof burden reviewers carry.

**Origins:** `LANG-UNSAFE`, `LANG-UNSAFE-ENCP`, `UNSAFE-NOUB` [ANSSI]; Item 16,
"Avoid writing unsafe code" [ER].

---

<!-- References -->

[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
[ER]: https://www.effective-rust.com/ "Effective Rust"
