# Prefer single-writer ownership for mutable hot state

Rule id: `build-performance/single-writer-hot-state`

**Rationale:** Single-writer designs are easier to reason about, reduce
synchronization costs, and avoid contention patterns that scale poorly under
load.

**Origins:** "The Principles of Mechanical Sympathy" [MS]. This source adds a
concurrency-design rule for hot mutable state that is not stated directly in the
higher-precedence guides.

---

<!-- References -->

[MS]: https://martinfowler.com/articles/mechanical-sympathy-principles.html "The Principles of Mechanical Sympathy"
