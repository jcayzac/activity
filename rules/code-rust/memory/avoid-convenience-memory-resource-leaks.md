# Do not leak memory or resources as a convenience

Rule id: `memory/avoid-convenience-memory-resource-leaks`

**Rationale:** Memory leaks, skipped destruction, and uninitialized memory are
security and correctness hazards, even when they do not immediately trigger
undefined behavior.

**Origins:** `MEM-NO-LEAK`, `MEM-FORGET`, `MEM-LEAK`, `MEM-UNINIT` [ANSSI].

---

<!-- References -->

[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
