# Use monotonic time for durations and timeouts

Rule id: `data-boundaries/monotonic-time-durations-timeouts`

**Rationale:** Wall-clock time can jump because of NTP, leap adjustments, or
operator action. Duration logic tied to it is fragile and can fail in ways that
are hard to reproduce.

**Origins:** [DDIA]. This source adds a time-semantics rule for distributed and
persistent systems that is not stated directly in the higher-precedence guides.

---

<!-- References -->

[DDIA]: https://dataintensive.net/ "Designing Data-Intensive Applications"
