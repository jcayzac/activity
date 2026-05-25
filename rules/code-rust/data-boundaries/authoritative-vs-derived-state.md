# Keep authoritative state distinct from derived state

Rule id: `data-boundaries/authoritative-vs-derived-state`

**Rationale:** When authoritative and derived state blur together, recovery,
debugging, and consistency reasoning become harder.

**Origins:** [DDIA]. This source adds a source-of-truth rule for derived state
that is not stated directly in the higher-precedence guides.

---

<!-- References -->

[DDIA]: https://dataintensive.net/ "Designing Data-Intensive Applications"
