# Translate data at every architectural boundary

Rule id: `boundaries/data-translation`

**Rationale:** Boundary translation prevents detail-owned types from leaking
inward. It keeps each side free to evolve its data shape for its own needs.

**Origins:** "The Clean Architecture" [CA]; "Common web application
architectures" [MS]; "Hexagonal architecture" [HEX].

---

<!-- References -->

[CA]: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html "The Clean Architecture"
[MS]: https://learn.microsoft.com/en-us/dotnet/architecture/modern-web-apps-azure/common-web-application-architectures "Common web application architectures"
[HEX]: https://alistair.cockburn.us/hexagonal-architecture/ "Hexagonal architecture the original 2005 article"
