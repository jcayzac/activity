# Run CI on the quality gates that define "done"

Rule id: `build-performance/ci-quality-gates`

**Rationale:** This makes "passing" objective and catches drift between local
habits and repository policy.

**Origins:** Item 32, "Set up a continuous integration (CI) system" [ER];
`DENV-LINTER`, `DENV-AUTOFIX` [ANSSI].

---

<!-- References -->

[ER]: https://www.effective-rust.com/ "Effective Rust"
[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
