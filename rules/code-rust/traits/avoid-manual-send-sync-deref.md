# Avoid manual `Send`, `Sync`, and `Deref` unless the type truly deserves them

Rule id: `traits/avoid-manual-send-sync-deref`

**Rationale:** These traits change aliasing, threading, and method resolution
behavior. A wrong impl can make safe code unsound or surprising.

**Origins:** `C-SEND-SYNC`, `C-DEREF`, `C-SMART-PTR` [API]; `LANG-SYNC-TRAITS`
[ANSSI]; "Deref Polymorphism" [RDP].

---

<!-- References -->

[API]: https://github.com/rust-lang/api-guidelines/blob/master/src/SUMMARY.md "The Rust API Guidelines"
[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
[RDP]: https://rust-unofficial.github.io/patterns/ "Rust Design Patterns"
