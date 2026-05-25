# Core logic must run without infrastructure

Rule id: `replacement-testing/infrastructure-free-core`

**Rationale:** If core logic cannot run without infrastructure, the
architecture has already leaked. Fast isolated tests are a consequence of good
boundaries, not a separate concern.

**Origins:** "Hexagonal architecture" [HEX]; "The Clean Architecture" [CA];
"Common web application architectures" [MS]; "Screaming Architecture" [SC].

---

<!-- References -->

[HEX]: https://alistair.cockburn.us/hexagonal-architecture/ "Hexagonal architecture the original 2005 article"
[CA]: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html "The Clean Architecture"
[MS]: https://learn.microsoft.com/en-us/dotnet/architecture/modern-web-apps-azure/common-web-application-architectures "Common web application architectures"
[SC]: https://blog.cleancoder.com/uncle-bob/2011/09/30/Screaming-Architecture.html "Screaming Architecture"
