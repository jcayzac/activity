# Do not clone just to satisfy the borrow checker

Rule id: `function-structure/avoid-clone-borrow-checker`

**Rationale:** Such clones often hide an ownership mistake, can desynchronize
state, and add unnecessary allocation or copying cost.

**Origins:** "Clone to satisfy the borrow checker" [RDP];
"`mem::{take(_), replace(_)}` to keep owned values in changed enums" [RDP].

---

<!-- References -->

[RDP]: https://rust-unofficial.github.io/patterns/ "Rust Design Patterns"
