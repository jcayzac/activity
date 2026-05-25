# Make integer overflow behavior explicit

Rule id: `errors/explicit-integer-overflow-behavior`

**Rationale:** Debug and release builds differ. Silent wraparound in release is
too easy to miss unless it is made explicit in the code.

**Origins:** `LANG-ARITH` [ANSSI].

---

<!-- References -->

[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
