# Treat frameworks, storage, and delivery mechanisms as edge details

Rule id: `policy-detail/edge-details`

**Rationale:** Delivery and integration mechanisms change more often than the
rules of the business. Keeping them at the edge limits churn and keeps the
center of the system stable.

**Origins:** "Screaming Architecture" [SC]; "The Clean Architecture" [CA];
"Hexagonal architecture" [HEX]; "Hexagonal architecture pattern" [AWS].

---

<!-- References -->

[SC]: https://blog.cleancoder.com/uncle-bob/2011/09/30/Screaming-Architecture.html "Screaming Architecture"
[CA]: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html "The Clean Architecture"
[HEX]: https://alistair.cockburn.us/hexagonal-architecture/ "Hexagonal architecture the original 2005 article"
[AWS]: https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/hexagonal-architecture.html "Hexagonal architecture pattern"
