# Batch work when the boundary cost dominates

Rule id: `build-performance/batch-expensive-boundaries`

**Rationale:** Boundary overhead often dominates the work itself. Batching
amortizes fixed costs and often improves cache, transport, and scheduling
behavior.

**Origins:** "The Principles of Mechanical Sympathy" [MS]. This source adds a
batching rule for expensive boundaries that is not stated directly in the
higher-precedence guides.

---

<!-- References -->

[MS]: https://martinfowler.com/articles/mechanical-sympathy-principles.html "The Principles of Mechanical Sympathy"
