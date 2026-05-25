# Require backward- and forward-compatible evolution for durable or cross-service data contracts

Rule id: `data-boundaries/compatible-data-contract-evolution`

**Rationale:** Long-lived systems rarely upgrade everywhere at once. Compatible
evolution prevents rolling deploys, reprocessing, and mixed-version fleets from
breaking each other.

**Origins:** [DDIA]. This source adds a schema-evolution rule for durable and
cross-service contracts that is not stated directly in the higher-precedence
guides.

---

<!-- References -->

[DDIA]: https://dataintensive.net/ "Designing Data-Intensive Applications"
