# Import only what keeps the code obvious

Rule id: `imports/minimal-obvious-imports`

**Rationale:** Over-importing hides where names come from, increases merge
conflicts, and makes the module header harder to skim.

**Origins:** `M-PRIV-USE`, `M-SINGLE-USE` [EP]; `Imports`,
`Ordering of imports`, `Merging/un-merging imports` [RSG]. These sources
diverge: Ed Page is stricter about minimal one-item imports, while the official
style guide is more permissive about merged imports. This rule follows Ed Page.

---

<!-- References -->

[EP]: https://raw.githubusercontent.com/epage/epage.github.io/refs/heads/source/blog/dev/rust-style.md "Ed Page's Rust Style Guide"
[RSG]: https://github.com/rust-lang/rust/blob/main/src/doc/style-guide/src/SUMMARY.md "The official Rust Style Guide"
