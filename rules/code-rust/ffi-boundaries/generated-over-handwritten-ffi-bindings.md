# Prefer generated FFI bindings over handwritten ones where practical

Rule id: `ffi-boundaries/generated-over-handwritten-ffi-bindings`

**Rationale:** Generators reduce transcription mistakes and drift in low-level
signatures, layouts, and constants. They do not remove the need for a reviewed
safe wrapper.

**Origins:** Item 35, "Prefer bindgen to manual FFI mappings" [ER];
`FFI-AUTOMATE`, `FFI-SAFEWRAPPING` [ANSSI].

---

<!-- References -->

[ER]: https://www.effective-rust.com/ "Effective Rust"
[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
