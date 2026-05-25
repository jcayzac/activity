# Keep runtime-specific concerns out of the core

Rule id: `policy-detail/runtime-free-core`

**Rationale:** In modern Rust, `tokio`, channels, schedulers, and
synchronization primitives can easily spread inward and become accidental
architecture. That couples policy to execution mechanics and makes testing and
reuse harder.

**Origins:** "Screaming Architecture" [SC]; "The Clean Architecture" [CA];
"Common web application architectures" [MS]. These sources classify framework
and delivery concerns as details; this guide makes that rule explicit for Rust
runtime choices.

---

<!-- References -->

[SC]: https://blog.cleancoder.com/uncle-bob/2011/09/30/Screaming-Architecture.html "Screaming Architecture"
[CA]: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html "The Clean Architecture"
[MS]: https://learn.microsoft.com/en-us/dotnet/architecture/modern-web-apps-azure/common-web-application-architectures "Common web application architectures"
