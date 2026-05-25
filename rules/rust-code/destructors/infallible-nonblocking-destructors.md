# Destructors must not fail, panic, or block

Rule id: `destructors/infallible-nonblocking-destructors`

**Rationale:** Drops can run during unwinding and during hard-to-debug control
paths. Failure or blocking in `Drop` turns cleanup into a second source of bugs.

**Origins:** `C-DTOR-FAIL`, `C-DTOR-BLOCK` [API]; `LANG-DROP`,
`LANG-DROP-NO-PANIC` [ANSSI]; Item 11, "Implement the Drop trait for RAII
patterns" [ER].

---

<!-- References -->

[API]: https://github.com/rust-lang/api-guidelines/blob/master/src/SUMMARY.md "The Rust API Guidelines"
[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
[ER]: https://www.effective-rust.com/ "Effective Rust"
