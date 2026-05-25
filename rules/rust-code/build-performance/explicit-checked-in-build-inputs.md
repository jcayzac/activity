# Keep build inputs explicit and checked in

Rule id: `build-performance/explicit-checked-in-build-inputs`

**Rationale:** Reproducible builds require reproducible inputs. Ambient
environment tweaks make review and debugging harder.

**Origins:** `DENV-CARGO-LOCK`, `DENV-CARGO-OPTS`, `DENV-CARGO-ENVVARS` [ANSSI].

---

<!-- References -->

[ANSSI]: https://github.com/ANSSI-FR/rust-guide/blob/master/src/en/SUMMARY.md "The ANSSI Rust Guide"
