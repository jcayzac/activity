# Define boundary traits on the consuming side

Rule id: `dependency-direction/consumer-owned-traits`

**Rationale:** The consumer owns the contract it needs. If the provider owns
the abstraction, policy starts depending on provider-shaped concepts instead of
its own needs.

**Origins:** "The Clean Architecture" [CA]; "Hexagonal architecture" [HEX];
"Inversion of Control Containers and the Dependency Injection pattern" [FOW].

---

<!-- References -->

[CA]: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html "The Clean Architecture"
[HEX]: https://alistair.cockburn.us/hexagonal-architecture/ "Hexagonal architecture the original 2005 article"
[FOW]: https://martinfowler.com/articles/injection.html "Inversion of Control Containers and the Dependency Injection pattern"
