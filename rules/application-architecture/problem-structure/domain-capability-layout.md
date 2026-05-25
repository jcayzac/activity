# Organize top-level code by domain and capability, not by framework

Rule id: `problem-structure/domain-capability-layout`

**Rationale:** Architecture should communicate use cases and business intent.
If the first thing the codebase says is "Axum app", "SQL project", or "Tokio
service", the structure is describing tools instead of the problem being
solved.

**Origins:** "Screaming Architecture" [SC]; "The Clean Architecture" [CA];
"Hexagonal architecture" [HEX]; "Hexagonal architecture pattern" [AWS]. These
sources align on purpose-first structure and framework independence.

---

<!-- References -->

[SC]: https://blog.cleancoder.com/uncle-bob/2011/09/30/Screaming-Architecture.html "Screaming Architecture"
[CA]: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html "The Clean Architecture"
[HEX]: https://alistair.cockburn.us/hexagonal-architecture/ "Hexagonal architecture the original 2005 article"
[AWS]: https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/hexagonal-architecture.html "Hexagonal architecture pattern"
