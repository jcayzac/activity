# Define application operations explicitly

Rule id: `boundaries/application-operations`

**Rationale:** An explicit application boundary gives the system a stable,
discoverable surface. It also gives tests, adapters, and reviewers a clear
place to look for orchestration and transactional behavior.

**Origins:** "Service Layer" [SL]; "Screaming Architecture" [SC];
"Hexagonal architecture" [HEX].

---

<!-- References -->

[SL]: https://martinfowler.com/eaaCatalog/serviceLayer.html "Service Layer"
[SC]: https://blog.cleancoder.com/uncle-bob/2011/09/30/Screaming-Architecture.html "Screaming Architecture"
[HEX]: https://alistair.cockburn.us/hexagonal-architecture/ "Hexagonal architecture the original 2005 article"
