# Make dependencies explicit in types and function signatures

Rule id: `composition/explicit-dependencies`

**Rationale:** Explicit dependencies are easier to review, test, replace, and
reason about. Hidden dependencies make architecture invisible.

**Origins:** [FOW]. Fowler allows both dependency injection and service
locator, but also notes that injection makes dependencies easier to see. This
guide adopts explicit dependency passing as the stricter house rule for policy
code.

---

<!-- References -->

[FOW]: https://martinfowler.com/articles/injection.html "Inversion of Control Containers and the Dependency Injection pattern"
