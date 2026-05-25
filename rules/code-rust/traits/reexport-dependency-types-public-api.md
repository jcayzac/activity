# Re-export dependency types that appear in your public API

Rule id: `traits/reexport-dependency-types-public-api`

**Rationale:** This makes the public dependency explicit and reduces friction
for users who would otherwise need to line up an exact dependency path and
version just to name a type that your API already exposes.

**Origins:** Item 24, "Re-export dependencies whose types appear in your API"
[ER].

---

<!-- References -->

[ER]: https://www.effective-rust.com/ "Effective Rust"
