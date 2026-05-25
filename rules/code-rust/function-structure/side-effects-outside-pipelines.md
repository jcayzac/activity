# Do not hide side effects inside expression pipelines

Rule id: `function-structure/side-effects-outside-pipelines`

**Rationale:** Readers skim pipelines as transformations. Hidden mutation in a
combinator is easy to miss.

**Origins:** `F-PURE-MUT`, `F-COMBINATOR` [EP].

---

<!-- References -->

[EP]: https://raw.githubusercontent.com/epage/epage.github.io/refs/heads/source/blog/dev/rust-style.md "Ed Page's Rust Style Guide"
