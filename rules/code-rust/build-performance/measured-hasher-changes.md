# Do not change hashers without a measured reason

Rule id: `build-performance/measured-hasher-changes`

**Rationale:** Hasher changes affect performance, determinism expectations, and
HashDoS resistance. They should be driven by measured hot spots, not fashion.

**Origins:** "Hashing" [RPB].

---

<!-- References -->

[RPB]: https://nnethercote.github.io/perf-book/ "The Rust Performance Book"
