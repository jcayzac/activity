# Test against supported dependency ranges when compatibility matters

Rule id: `build-performance/test-supported-dependency-ranges`

**Rationale:** A wide dependency constraint is only meaningful if it is actually
exercised. Otherwise, compatibility can silently regress while `Cargo.toml`
still claims it.

**Origins:** Item 21, "Understand what semantic versioning promises" [ER]; Item
31, "Test libraries against different dependency versions" [ER].

---

<!-- References -->

[ER]: https://www.effective-rust.com/ "Effective Rust"
