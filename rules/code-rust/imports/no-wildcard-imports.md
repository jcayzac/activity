# Avoid wildcard imports in production code

Rule id: `imports/no-wildcard-imports`

**Rationale:** Wildcard imports hide where names come from, make accidental name
capture easier, and make review harder when APIs change.

**Origins:** Item 23, "Avoid wildcard imports" [ER]; `M-PRIV-USE`,
`M-SINGLE-USE` [EP].

---

<!-- References -->

[ER]: https://www.effective-rust.com/ "Effective Rust"
[EP]: https://raw.githubusercontent.com/epage/epage.github.io/refs/heads/source/blog/dev/rust-style.md "Ed Page's Rust Style Guide"
