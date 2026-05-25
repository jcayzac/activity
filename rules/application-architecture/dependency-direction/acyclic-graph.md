# Keep the dependency graph acyclic

Rule id: `dependency-direction/acyclic-graph`

**Rationale:** Cycles destroy architectural direction. They make independent
reasoning, testing, release planning, and replacement harder because no part of
the cycle can move safely on its own.

**Origins:** [PAR]; [CA]; [MS]. These sources all assume one-way dependency
structures. This guide makes acyclicity explicit because Rust crates and
modules can and should enforce it mechanically.

---

<!-- References -->

[PAR]: https://sunnyday.mit.edu/16.355/parnas-criteria.html "On the Criteria To Be Used in Decomposing Systems into Modules"
[CA]: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html "The Clean Architecture"
[MS]: https://learn.microsoft.com/en-us/dotnet/architecture/modern-web-apps-azure/common-web-application-architectures "Common web application architectures"
