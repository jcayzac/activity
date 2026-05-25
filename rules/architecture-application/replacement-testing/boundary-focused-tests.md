# Test policy and adapters at their own boundaries

Rule id: `replacement-testing/boundary-focused-tests`

**Rationale:** Boundary-focused tests reinforce the architecture instead of
reaching through it. They also keep failures local and easier to diagnose.

**Origins:** "Quality by design" [AWSQ]; "Hexagonal architecture pattern"
[AWS]; "Service Layer" [SL].

---

<!-- References -->

[AWSQ]: https://docs.aws.amazon.com/prescriptive-guidance/latest/hexagonal-architectures/improve-software-quality.html "Quality by design"
[AWS]: https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/hexagonal-architecture.html "Hexagonal architecture pattern"
[SL]: https://martinfowler.com/eaaCatalog/serviceLayer.html "Service Layer"
