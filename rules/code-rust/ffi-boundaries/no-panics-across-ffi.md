# Do not let panics cross the FFI boundary

Rule id: `ffi-boundaries/no-panics-across-ffi`

**Rationale:** Crossing language runtimes with unwinding is fragile and
difficult to reason about, even when technically supported.

**Origins:** `FFI-NOPANIC` [ANSSI]; Item 18, "Don't panic" [ER]; Item 34,
"Control what crosses FFI boundaries" [ER].

---

<!-- References -->

[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
[ER]: https://www.effective-rust.com/ "Effective Rust"
