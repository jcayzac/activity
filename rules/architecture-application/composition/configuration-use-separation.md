# Separate configuration from use

Rule id: `composition/configuration-use-separation`

**Rationale:** When selection and use are mixed, every caller starts carrying
construction logic and environment knowledge. That duplicates wiring and hides
real dependencies.

**Origins:** "Inversion of Control Containers and the Dependency Injection
pattern" [FOW].

---

<!-- References -->

[FOW]: https://martinfowler.com/articles/injection.html "Inversion of Control Containers and the Dependency Injection pattern"
