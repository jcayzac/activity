# Keep memory access patterns predictable in hot code

Rule id: `build-performance/predictable-memory-access`

**Rationale:** Modern CPUs are fast at arithmetic and slow at waiting for
memory. Code that defeats caches and prefetching can lose more to memory stalls
than it gains from local micro-optimizations.

**Origins:** "The Principles of Mechanical Sympathy" [MS]. This source adds a
hardware-aware hot-path rule that is not stated directly in the higher-
precedence guides.

---

<!-- References -->

[MS]: https://martinfowler.com/articles/mechanical-sympathy-principles.html "The Principles of Mechanical Sympathy"
