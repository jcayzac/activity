# Use only simple visibility levels

Rule id: `visibility-layout/simple-visibility-levels`

**Rationale:** Overly precise visibility usually adds friction to refactors
without adding much real encapsulation. Most code only needs module-private,
crate-private, or public.

**Origins:** `P-VISIBILITY` [EP]; `C-STRUCT-PRIVATE` [API]; Item 22, "Minimize
visibility" [ER].

---

<!-- References -->

[EP]: https://raw.githubusercontent.com/epage/epage.github.io/refs/heads/source/blog/dev/rust-style.md "Ed Page's Rust Style Guide"
[API]: https://github.com/rust-lang/api-guidelines/blob/master/src/SUMMARY.md "The Rust API Guidelines"
[ER]: https://www.effective-rust.com/ "Effective Rust"
