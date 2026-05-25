# Keep domain rules, application orchestration, and adapters distinct

Rule id: `change-boundaries/policy-orchestration-adapters`

**Rationale:** These concerns change for different reasons and are tested in
different ways. Distinguishing them keeps business rules rich, orchestration
explicit, and infrastructure replaceable.

**Origins:** "The Clean Architecture" [CA]; "Service Layer" [SL]; "Common web
application architectures" [MS]; "Hexagonal architecture" [HEX].

---

<!-- References -->

[CA]: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html "The Clean Architecture"
[SL]: https://martinfowler.com/eaaCatalog/serviceLayer.html "Service Layer"
[MS]: https://learn.microsoft.com/en-us/dotnet/architecture/modern-web-apps-azure/common-web-application-architectures "Common web application architectures"
[HEX]: https://alistair.cockburn.us/hexagonal-architecture/ "Hexagonal architecture the original 2005 article"
