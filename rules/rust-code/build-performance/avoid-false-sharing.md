# Avoid false sharing in hot mutable state

Rule id: `build-performance/avoid-false-sharing`

**Rationale:** False sharing creates coherence traffic even when threads touch
different variables. That turns independent work into avoidable contention.

**Origins:** "The Principles of Mechanical Sympathy" [MS]. This source adds a
cache-line rule for contended hot state that is not stated directly in the
higher-precedence guides.

---

<!-- References -->

[MS]: https://martinfowler.com/articles/mechanical-sympathy-principles.html "The Principles of Mechanical Sympathy"
