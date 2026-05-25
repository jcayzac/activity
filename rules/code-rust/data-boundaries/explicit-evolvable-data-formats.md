# Make wire and stored data formats explicit, schema-driven, and evolvable

Rule id: `data-boundaries/explicit-evolvable-data-formats`

**Rationale:** Durable and cross-process data outlives individual code
revisions. Ad hoc formats make migration, debugging, interoperability, and
recovery harder than they need to be.

**Origins:** [DDIA]. This source adds a durable-data contract rule that is not
stated directly in the higher-precedence guides.

---

<!-- References -->

[DDIA]: https://dataintensive.net/ "Designing Data-Intensive Applications"
