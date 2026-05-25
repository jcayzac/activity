# Document non-obvious optimizations with the measurement that justified them

Rule id: `build-performance/document-measured-optimizations`

**Rationale:** Measurement-backed comments keep optimized code reviewable and
reduce the risk that later cleanup removes a necessary optimization or preserves
an obsolete one.

**Origins:** "General Tips" [RPB].

---

<!-- References -->

[RPB]: https://nnethercote.github.io/perf-book/ "The Rust Performance Book"
