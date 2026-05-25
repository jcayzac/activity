# Keep the dependency graph intentional

Rule id: `build-performance/intentional-dependency-graph`

**Rationale:** Every dependency expands the maintenance, security, and build
surface. Graph drift is easy to miss because it often happens transitively.

**Origins:** Item 25, "Manage your dependency graph" [ER]; Item 26, "Be wary of
feature creep" [ER]; `LIBS-VETTING-DIRECT` [ANSSI].

---

<!-- References -->

[ER]: https://www.effective-rust.com/ "Effective Rust"
[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
