# Give each crate and module one primary reason to change

Rule id: `change-boundaries/single-change-reason`

**Rationale:** Modules become maintainable when their internal parts change
together. Mixing unrelated concerns creates surprise regressions and ties
unrelated work streams to the same code.

**Origins:** [PAR]; [SRP]. The SRP article explicitly defines a module as having
one reason to change and traces that idea back to Parnas's change-based
decomposition.

---

<!-- References -->

[PAR]: https://sunnyday.mit.edu/16.355/parnas-criteria.html "On the Criteria To Be Used in Decomposing Systems into Modules"
[SRP]: https://blog.cleancoder.com/uncle-bob/2014/05/08/SingleReponsibilityPrinciple.html "The Single Responsibility Principle"
