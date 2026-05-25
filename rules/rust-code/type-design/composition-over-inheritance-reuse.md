# Prefer composition over inheritance-style reuse

Rule id: `type-design/composition-over-inheritance-reuse`

**Rationale:** Composition keeps responsibilities local and explicit, avoids
surprising method lookup, and usually works better with Rust's ownership model.

**Origins:** "Compose Structs" [RDP]; "Deref Polymorphism" [RDP]. These sources
frame composition as the preferred structural pattern and deref-based
inheritance emulation as an anti-pattern.

---

<!-- References -->

[RDP]: https://rust-unofficial.github.io/patterns/ "Rust Design Patterns"
