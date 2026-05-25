# Be deliberate about features

Rule id: `cargo-manifest/deliberate-features`

**Rationale:** Feature creep complicates testing, review, compatibility, and
user understanding. A smaller, more intentional feature surface is easier to
support.

**Origins:** Item 26, "Be wary of feature creep" [ER]; `C-FEATURE` [API].

---

<!-- References -->

[ER]: https://www.effective-rust.com/ "Effective Rust"
[API]: https://github.com/rust-lang/api-guidelines/blob/master/src/SUMMARY.md "The Rust API Guidelines"
