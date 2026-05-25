# Do not create catch-all shared crates or modules

Rule id: `problem-structure/catch-all-shared-modules`

**Rationale:** Catch-all shared modules hide architecture drift. They mix
reasons to change, accumulate incidental coupling, and make it impossible to
see real subsystem boundaries.

**Origins:** "On the Criteria To Be Used in Decomposing Systems into Modules"
[PAR]; "The Single Responsibility Principle" [SRP]; "Screaming Architecture"
[SC]. This guide turns their cohesion and domain-centered structure into an
explicit ban on dumping grounds.

---

<!-- References -->

[PAR]: https://sunnyday.mit.edu/16.355/parnas-criteria.html "On the Criteria To Be Used in Decomposing Systems into Modules"
[SRP]: https://blog.cleancoder.com/uncle-bob/2014/05/08/SingleReponsibilityPrinciple.html "The Single Responsibility Principle"
[SC]: https://blog.cleancoder.com/uncle-bob/2011/09/30/Screaming-Architecture.html "Screaming Architecture"
