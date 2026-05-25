# Prefer RAII guards for scoped access to resources or invariants

Rule id: `destructors/raii-guards-scoped-access`

**Rationale:** RAII guards keep acquisition and release coupled to lexical
scope, which prevents forgotten cleanup and makes misuse harder to express.

**Origins:** "RAII Guards" [RDP]; `C-DTOR-FAIL`, `C-DTOR-BLOCK` [API];
`LANG-DROP`, `LANG-DROP-NO-PANIC` [ANSSI].

---

<!-- References -->

[RDP]: https://rust-unofficial.github.io/patterns/ "Rust Design Patterns"
[API]: https://github.com/rust-lang/api-guidelines/blob/master/src/SUMMARY.md "The Rust API Guidelines"
[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
