# Do not rely on `Drop` for security-sensitive cleanup

Rule id: `memory/no-drop-security-cleanup`

**Rationale:** `Drop` is not guaranteed to run in every failure mode, and
reference cycles can suppress it entirely.

**Origins:** `LANG-DROP-SEC`, `LANG-DROP-NO-CYCLE` [ANSSI].

---

<!-- References -->

[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
