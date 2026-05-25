# Do not encode meaning in `bool`, `Option`, or raw integers when a real type would do

Rule id: `type-design/domain-types-over-bool-option-integers`

**Rationale:** Domain types make call sites readable, encode invariants, and
leave room for future growth.

**Origins:** `C-NEWTYPE`, `C-CUSTOM-TYPE`, `C-BITFLAG`, `C-BUILDER` [API];
"Newtype" [RDP]; Item 6, "Embrace the newtype pattern" [ER]; Item 7, "Use
builders for complex types" [ER].

---

<!-- References -->

[API]: https://github.com/rust-lang/api-guidelines/blob/master/src/SUMMARY.md "The Rust API Guidelines"
[RDP]: https://rust-unofficial.github.io/patterns/ "Rust Design Patterns"
[ER]: https://www.effective-rust.com/ "Effective Rust"
