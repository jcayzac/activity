# Keep technical cross-cutting concerns out of business policy

Rule id: `cross-cutting/technical-policy-separation`

**Rationale:** Cross-cutting infrastructure is important, but it is not the
domain. When it captures business rules, those rules become fragmented and
harder to audit.

**Origins:** "The Clean Architecture" [CA]; "Common web application
architectures" [MS]; "Layering Principles" [LAY].

---

<!-- References -->

[CA]: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html "The Clean Architecture"
[MS]: https://learn.microsoft.com/en-us/dotnet/architecture/modern-web-apps-azure/common-web-application-architectures "Common web application architectures"
[LAY]: https://martinfowler.com/bliki/LayeringPrinciples.html "Layering Principles"
