# Keep signatures and type layouts easy to scan

Rule id: `formatting/scannable-signatures-type-layouts`

**Rationale:** Wide one-line declarations are hard to diff and hard to skim. One
logical unit per line keeps shape and churn visible.

**Origins:** `Function definitions`, `Structs and Unions`, `Enums`,
`Tuples and tuple structs`, `Types and Bounds` [RSG]; `M-TYPE-ASSOC` [EP].

---

<!-- References -->

[RSG]: https://github.com/rust-lang/rust/blob/main/src/doc/style-guide/src/SUMMARY.md "The official Rust Style Guide"
[EP]: https://raw.githubusercontent.com/epage/epage.github.io/refs/heads/source/blog/dev/rust-style.md "Ed Page's Rust Style Guide"
