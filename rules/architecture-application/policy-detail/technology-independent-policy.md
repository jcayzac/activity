# Keep business rules independent of technology

Rule id: `policy-detail/technology-independent-policy`

**Rationale:** Business rules outlive implementation choices. When policy code
depends on details, replacing a database, web framework, or cloud integration
becomes a rewrite instead of a substitution.

**Origins:** "The Clean Architecture" [CA]; "Common web application
architectures" [MS]; "Hexagonal architecture" [HEX]; "The Onion Architecture"
[OA]. These sources all require business logic to stay independent of
infrastructure details.

---

<!-- References -->

[CA]: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html "The Clean Architecture"
[MS]: https://learn.microsoft.com/en-us/dotnet/architecture/modern-web-apps-azure/common-web-architecture-applications "Common web application architectures"
[HEX]: https://alistair.cockburn.us/hexagonal-architecture/ "Hexagonal architecture the original 2005 article"
[OA]: https://jeffreypalermo.com/2008/07/the-onion-architecture-part-1/ "The Onion Architecture : part 1"
