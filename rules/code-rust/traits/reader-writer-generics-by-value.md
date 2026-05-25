# Reader/writer generics take ownership by value

Rule id: `traits/reader-writer-generics-by-value`

**Rationale:** This is the standard Rust pattern, and it stays ergonomic because
`&mut R` and `&mut W` also implement the relevant traits.

**Origins:** `C-RW-VALUE` [API].

---

<!-- References -->

[API]: https://github.com/rust-lang/api-guidelines/blob/master/src/SUMMARY.md "The Rust API Guidelines"
