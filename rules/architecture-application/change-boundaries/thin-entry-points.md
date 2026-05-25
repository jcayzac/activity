# Keep entry points thin

Rule id: `change-boundaries/thin-entry-points`

**Rationale:** Entry points are adapter code. When they accumulate policy, the
application becomes difficult to test outside the chosen delivery mechanism and
business behavior becomes fragmented across many edges.

**Origins:** "The Clean Architecture" [CA]; "Screaming Architecture" [SC];
"Service Layer" [SL].

---

<!-- References -->

[CA]: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html "The Clean Architecture"
[SC]: https://blog.cleancoder.com/uncle-bob/2011/09/30/Screaming-Architecture.html "Screaming Architecture"
[SL]: https://martinfowler.com/eaaCatalog/serviceLayer.html "Service Layer"
