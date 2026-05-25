# If you need a macro, make it read like Rust

Rule id: `macro-style/rust-like-macros`

**Rationale:** Macros are part of the language surface. If they feel unlike
Rust, users misread them and tools handle them less well.

**Origins:** `C-EVOCATIVE`, `C-MACRO-ATTR`, `C-ANYWHERE`, `C-MACRO-VIS`,
`C-MACRO-TY` [API]; Item 28, "Use macros judiciously" [ER].

---

<!-- References -->

[API]: https://github.com/rust-lang/api-guidelines/blob/master/src/SUMMARY.md "The Rust API Guidelines"
[ER]: https://www.effective-rust.com/ "Effective Rust"
