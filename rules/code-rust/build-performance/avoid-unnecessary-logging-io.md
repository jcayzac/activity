# Avoid unnecessary logging and I/O work in non-hot paths

Rule id: `build-performance/avoid-unnecessary-logging-io`

**Rationale:** Logging and I/O often sit on frequent paths. Small per-call
overheads compound quickly, and they are easy to miss in review because the code
looks innocuous.

**Origins:** "Logging and Debugging" [RPB]; "I/O" [RPB].

---

<!-- References -->

[RPB]: https://nnethercote.github.io/perf-book/ "The Rust Performance Book"
