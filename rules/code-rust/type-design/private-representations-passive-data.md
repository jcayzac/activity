# Keep representations private unless the type is intentionally passive data

Rule id: `type-design/private-representations-passive-data`

**Rationale:** Public fields and unsealed extension points lock in
representation choices and make future evolution harder.

**Origins:** `C-STRUCT-PRIVATE`, `C-SEALED`, `C-NEWTYPE-HIDE` [API];
"Object-Based APIs" [RDP]; "Type Consolidation into Wrappers" [RDP]; Item 22,
"Minimize visibility" [ER].

---

<!-- References -->

[API]: https://github.com/rust-lang/api-guidelines/blob/master/src/SUMMARY.md "The Rust API Guidelines"
[RDP]: https://rust-unofficial.github.io/patterns/ "Rust Design Patterns"
[ER]: https://www.effective-rust.com/ "Effective Rust"
