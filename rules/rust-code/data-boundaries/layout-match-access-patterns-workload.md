# Choose storage and in-memory layout to match access patterns and workload

Rule id: `data-boundaries/layout-match-access-patterns-workload`

**Rationale:** Layout is part of behavior. A representation that matches the
dominant workload is often simpler and faster than a one-size-fits-all design.

**Origins:** [DDIA]; "The Principles of Mechanical Sympathy" [MS]. These sources
align on choosing layouts around real access patterns rather than abstract
uniformity.

---

<!-- References -->

[DDIA]: https://dataintensive.net/ "Designing Data-Intensive Applications"
[MS]: https://martinfowler.com/articles/mechanical-sympathy-principles.html "The Principles of Mechanical Sympathy"
