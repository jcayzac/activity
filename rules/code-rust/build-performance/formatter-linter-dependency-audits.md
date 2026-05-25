# Run the formatter, the linter, and dependency audits regularly

Rule id: `build-performance/formatter-linter-dependency-audits`

**Rationale:** Mechanical consistency, lint feedback, and dependency hygiene
catch a large class of issues before review.

**Origins:** `DENV-FORMAT`, `DENV-LINTER`, `DENV-AUTOFIX`, `LIBS-OUTDATED`,
`LIBS-AUDIT`, `LIBS-VETTING-DIRECT` [ANSSI]; Item 29, "Listen to Clippy" [ER];
Item 31, "Take advantage of the tooling ecosystem" [ER].

---

<!-- References -->

[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
[ER]: https://www.effective-rust.com/ "Effective Rust"
