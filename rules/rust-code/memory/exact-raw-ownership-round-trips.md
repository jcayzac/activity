# Make raw ownership round-trips exact

Rule id: `memory/exact-raw-ownership-round-trips`

**Rationale:** Raw ownership transfer is all-or-nothing. Ambiguity here becomes
leaks, double frees, or use-after-free.

**Origins:** `MEM-INTOFROMRAWALWAYS`, `MEM-INTOFROMRAWONLY` [ANSSI].

---

<!-- References -->

[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
