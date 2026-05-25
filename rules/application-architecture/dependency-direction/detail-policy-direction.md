# Dependencies must point from volatile details toward stable policy

Rule id: `dependency-direction/detail-policy-direction`

**Rationale:** Architectural boundaries only work if dependency direction is
enforced. Otherwise infrastructure choices leak inward and the intended layering
or ports-and-adapters structure collapses.

**Origins:** "The Clean Architecture" [CA]; "Common web application
architectures" [MS]; "The Onion Architecture" [OA]; "On the Criteria To Be
Used in Decomposing Systems into Modules" [PAR].

---

<!-- References -->

[CA]: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html "The Clean Architecture"
[MS]: https://learn.microsoft.com/en-us/dotnet/architecture/modern-web-apps-azure/common-web-application-architectures "Common web application architectures"
[OA]: https://jeffreypalermo.com/2008/07/the-onion-architecture-part-1/ "The Onion Architecture : part 1"
[PAR]: https://sunnyday.mit.edu/16.355/parnas-criteria.html "On the Criteria To Be Used in Decomposing Systems into Modules"
