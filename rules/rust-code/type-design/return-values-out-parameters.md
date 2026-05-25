# Return values instead of out-parameters

Rule id: `type-design/return-values-out-parameters`

**Rationale:** Return values are clearer at the call site, compose better, and
match normal Rust expectations.

**Origins:** `C-NO-OUT` [API].

---

<!-- References -->

[API]: https://github.com/rust-lang/api-guidelines/blob/master/src/SUMMARY.md "The Rust API Guidelines"
