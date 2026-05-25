# Wire concrete implementations in a composition root

Rule id: `composition/composition-root`

**Rationale:** Construction is an infrastructure concern. Keeping it at the edge
prevents policy code from knowing how concrete dependencies are chosen or
assembled.

**Origins:** "Inversion of Control Containers and the Dependency Injection
pattern" [FOW]; "Common web application architectures" [MS]; "The Clean
Architecture" [CA].

---

<!-- References -->

[FOW]: https://martinfowler.com/articles/injection.html "Inversion of Control Containers and the Dependency Injection pattern"
[MS]: https://learn.microsoft.com/en-us/dotnet/architecture/modern-web-apps-azure/common-web-architecture-applications "Common web application architectures"
[CA]: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html "The Clean Architecture"
