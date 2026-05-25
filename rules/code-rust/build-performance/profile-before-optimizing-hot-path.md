# Profile before you optimize, and optimize the hot path first

Rule id: `build-performance/profile-before-optimizing-hot-path`

**Rationale:** Optimized code is usually more complex. The biggest wins often
come from higher-level changes, and only hot code is worth making less obvious.

**Origins:** Item 20, "Avoid the temptation to over-optimize" [ER]; "Profiling"
[RPB]; "General Tips" [RPB]; "The Principles of Mechanical Sympathy" [MS].

---

<!-- References -->

[ER]: https://www.effective-rust.com/ "Effective Rust"
[RPB]: https://nnethercote.github.io/perf-book/ "The Rust Performance Book"
[MS]: https://martinfowler.com/articles/mechanical-sympathy-principles.html "The Principles of Mechanical Sympathy"
