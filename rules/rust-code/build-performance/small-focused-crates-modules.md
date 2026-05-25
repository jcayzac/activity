# Prefer small crates and modules with focused responsibilities

Rule id: `build-performance/small-focused-crates-modules`

**Rationale:** Smaller compilation and responsibility units are easier to
understand, test, reuse, and review. They also make architectural boundaries
explicit. Over-splitting is still a cost, so the split needs a real boundary.

**Origins:** "Prefer small crates" [RDP]. This source adds a structural rule
that is not stated directly in the higher-precedence guides.

---

<!-- References -->

[RDP]: https://rust-unofficial.github.io/patterns/ "Rust Design Patterns"
