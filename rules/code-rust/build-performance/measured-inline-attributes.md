# Use inline attributes sparingly and only after measurement

Rule id: `build-performance/measured-inline-attributes`

**Rationale:** Inlining can improve runtime performance, but it can also worsen
compile times and code size. It is an optimization tool, not a default style.

**Origins:** "Inlining" [RPB].

---

<!-- References -->

[RPB]: https://nnethercote.github.io/perf-book/ "The Rust Performance Book"
