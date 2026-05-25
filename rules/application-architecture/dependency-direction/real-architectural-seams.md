# Introduce ports and adapters only at real architectural seams

Rule id: `dependency-direction/real-architectural-seams`

**Rationale:** Abstractions also have carrying cost. Architecture is about
intentional seams, not ceremonial indirection.

**Origins:** "Hexagonal architecture pattern" [AWS]; "Hexagonal architecture"
[HEX]. AWS explicitly notes the maintenance overhead of extra adapters when no
real seam exists; this guide adopts that as a house rule.

---

<!-- References -->

[AWS]: https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/hexagonal-architecture.html "Hexagonal architecture pattern"
[HEX]: https://alistair.cockburn.us/hexagonal-architecture/ "Hexagonal architecture the original 2005 article"
