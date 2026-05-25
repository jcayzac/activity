# Prefer object-based wrapper APIs at FFI boundaries

Rule id: `ffi-boundaries/object-based-ffi-wrappers`

**Rationale:** Object-based wrappers reduce the unsafe surface area, centralize
ownership and lifetime rules, and make it easier to keep the safe Rust API and
the foreign API aligned.

**Origins:** "Object-Based APIs" [RDP]; "Type Consolidation into Wrappers"
[RDP]; `FFI-SAFEWRAPPING` [ANSSI].

---

<!-- References -->

[RDP]: https://rust-unofficial.github.io/patterns/ "Rust Design Patterns"
[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
