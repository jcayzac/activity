# Choose the least complex design that preserves the rules

Rule id: `meta/least-complex-design`

**Rationale:** Indirection is a cost, not a virtue. The goal is clear
responsibility and replaceable details, not architectural ceremony.

**Origins:** "Hexagonal architecture pattern" [AWS]. AWS explicitly notes the
maintenance overhead of extra adapters; this guide generalizes that into a
least-complexity rule.

---

<!-- References -->

[AWS]: https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/hexagonal-architecture.html "Hexagonal architecture pattern"
