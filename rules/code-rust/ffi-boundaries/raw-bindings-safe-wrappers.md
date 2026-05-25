# Split FFI into raw bindings and safe wrappers

Rule id: `ffi-boundaries/raw-bindings-safe-wrappers`

**Rationale:** Raw FFI reflects foreign unsafety; the safe wrapper is where
Rust-side invariants are restored and reviewed.

**Origins:** `FFI-SAFEWRAPPING` [ANSSI]; "Object-Based APIs" [RDP]; "Type
Consolidation into Wrappers" [RDP].

---

<!-- References -->

[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
[RDP]: https://rust-unofficial.github.io/patterns/ "Rust Design Patterns"
