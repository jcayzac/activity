# Keep adapter-specific traits, derives, and annotations out of core types

Rule id: `boundaries/adapter-metadata-isolation`

**Rationale:** In Rust, derives and attributes are often architectural
dependencies in disguise. Once core types are forced to serve transport or
persistence frameworks directly, the core stops owning its own model.

**Origins:** "The Clean Architecture" [CA]; "Hexagonal architecture" [HEX];
"Common web application architectures" [MS]. These sources require inward
independence from framework data formats; this guide makes the Rust-specific
implication explicit.

---

<!-- References -->

[CA]: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html "The Clean Architecture"
[HEX]: https://alistair.cockburn.us/hexagonal-architecture/ "Hexagonal architecture the original 2005 article"
[MS]: https://learn.microsoft.com/en-us/dotnet/architecture/modern-web-apps-azure/common-web-architecture-applications "Common web application architectures"
