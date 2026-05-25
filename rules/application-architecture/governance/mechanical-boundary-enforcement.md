# Enforce important boundaries mechanically

Rule id: `governance/mechanical-boundary-enforcement`

**Rationale:** Architecture that is only documented will eventually be violated.
Rust gives strong tools for turning intended boundaries into compile-time and
review-time constraints.

**Origins:** [CA]; [MS]; [ADR]. This guide adds a Rust-specific enforcement rule
to make the architectural constraints durable.

---

<!-- References -->

[CA]: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html "The Clean Architecture"
[MS]: https://learn.microsoft.com/en-us/dotnet/architecture/modern-web-apps-azure/common-web-application-architectures "Common web application architectures"
[ADR]: https://docs.aws.amazon.com/prescriptive-guidance/latest/architectural-decision-records/welcome.html "Using architectural decision records to streamline technical decision-making for a software development project"
