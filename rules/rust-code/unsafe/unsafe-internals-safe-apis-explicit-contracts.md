# Encapsulate unsafe internals behind safe APIs or explicit unsafe contracts

Rule id: `unsafe/unsafe-internals-safe-apis-explicit-contracts`

**Rationale:** The entire point of encapsulation is to keep undefined behavior
out of safe call sites.

**Origins:** `LANG-UNSAFE-ENCP`, `unsafe` marking vs unlocking [ANSSI];
`C-FAILURE` [API]; "Contain unsafety in small modules" [RDP].

---

<!-- References -->

[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
[API]: https://github.com/rust-lang/api-guidelines/blob/master/src/SUMMARY.md "The Rust API Guidelines"
[RDP]: https://rust-unofficial.github.io/patterns/ "Rust Design Patterns"
