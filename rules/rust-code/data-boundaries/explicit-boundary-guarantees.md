# Make consistency, ordering, and durability guarantees explicit at boundaries

Rule id: `data-boundaries/explicit-boundary-guarantees`

**Rationale:** Boundary bugs often come from mismatched assumptions, not from
local code defects. Explicit guarantees give reviewers something concrete to
check against the implementation.

**Origins:** [DDIA]. This source adds a distributed-boundary contract rule that
is not stated directly in the higher-precedence guides.

---

<!-- References -->

[DDIA]: https://dataintensive.net/ "Designing Data-Intensive Applications"
