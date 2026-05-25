# Treat foreign values as untrusted until checked

Rule id: `ffi-boundaries/validate-foreign-values`

**Rationale:** Foreign code cannot be assumed to uphold Rust's validity rules.
Validation must happen at the boundary or before first use.

**Origins:** `FFI-CKNONROBUST`, `FFI-CK-PTR-VALID`, `FFI-INPUT-PTR`,
`FFI-CK-INPUT-REF-VALID`, `FFI-MARKEDFUNPTR`, `FFI-CKFUNPTR`, `FFI-NOENUM`
[ANSSI]; Item 34, "Control what crosses FFI boundaries" [ER].

---

<!-- References -->

[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
[ER]: https://www.effective-rust.com/ "Effective Rust"
