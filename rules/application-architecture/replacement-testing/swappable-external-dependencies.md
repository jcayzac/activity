# Every external dependency must be swappable

Rule id: `replacement-testing/swappable-external-dependencies`

**Rationale:** Replaceability is the practical meaning of low coupling. If a
database, API client, queue, clock, or file system cannot be swapped, then it
still owns part of the application.

**Origins:** "Hexagonal architecture" [HEX]; "Hexagonal architecture pattern"
[AWS]; "The Clean Architecture" [CA].

---

<!-- References -->

[HEX]: https://alistair.cockburn.us/hexagonal-architecture/ "Hexagonal architecture the original 2005 article"
[AWS]: https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/hexagonal-architecture.html "Hexagonal architecture pattern"
[CA]: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html "The Clean Architecture"
