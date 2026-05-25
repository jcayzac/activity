# Normal failure is `Result`, not panic

Rule id: `errors/result-not-panic`

**Rationale:** Panics are not an error-reporting API. They bypass caller choice,
complicate recovery, and can become process aborts.

**Origins:** `C-GOOD-ERR` [API]; `LANG-LIMIT-PANIC`, `LANG-LIMIT-PANIC-SRC`,
`LANG-ARRINDEXING` [ANSSI]; Item 18, "Don't panic" [ER].

---

<!-- References -->

[API]: https://github.com/rust-lang/api-guidelines/blob/master/src/SUMMARY.md "The Rust API Guidelines"
[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
[ER]: https://www.effective-rust.com/ "Effective Rust"
