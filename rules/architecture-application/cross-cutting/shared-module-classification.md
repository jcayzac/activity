# Shared modules must declare whether they are policy or infrastructure

Rule id: `cross-cutting/shared-module-classification`

**Rationale:** Mixed shared modules quietly become the place where architecture
erosion hides. Declared ownership keeps review and dependency direction honest.

**Origins:** [PAR]; [SRP]; [CA]. This guide makes the required classification
explicit so shared code cannot become an escape hatch around the other rules.

---

<!-- References -->

[PAR]: https://sunnyday.mit.edu/16.355/parnas-criteria.html "On the Criteria To Be Used in Decomposing Systems into Modules"
[SRP]: https://blog.cleancoder.com/uncle-bob/2014/05/08/SingleReponsibilityPrinciple.html "The Single Responsibility Principle"
[CA]: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html "The Clean Architecture"
