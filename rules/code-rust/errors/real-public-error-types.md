# Public library errors must have real types

Rule id: `errors/real-public-error-types`

**Rationale:** Specific error types preserve context, compose with `?`, and stay
usable across threads and wrappers. Type-erased library errors hide structure
the caller may need.

**Origins:** `C-GOOD-ERR` [API]; `LANG-ERRWRAP` [ANSSI]; Item 4, "Prefer
idiomatic Error types" [ER].

---

<!-- References -->

[API]: https://github.com/rust-lang/api-guidelines/blob/master/src/SUMMARY.md "The Rust API Guidelines"
[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
[ER]: https://www.effective-rust.com/ "Effective Rust"
