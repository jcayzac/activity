# Use only C-compatible types at the boundary

Rule id: `ffi-boundaries/c-compatible-ffi-types`

**Rationale:** Layout mismatches at the boundary are instant undefined behavior.

**Origins:** `FFI-CTYPE`, `FFI-TCONS`, `FFI-PFTYPE` [ANSSI]; Item 34, "Control
what crosses FFI boundaries" [ER].

---

<!-- References -->

[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
[ER]: https://www.effective-rust.com/ "Effective Rust"
