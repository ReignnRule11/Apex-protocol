# ADR-0054 — process-termination recovery tests for finality journal

- **Status:** Accepted for isolated research only; not production adoption.
- **Date:** 2026-11-08
- **Context:** Phase 20G2C6 provides an isolated redb journal and orderly close/reopen readback tests. Those tests do not exercise the case where the process terminates while a write transaction is staged or immediately after commit returns but before the caller acknowledges finality.
- **Decision:** Add test-only hooks around the journal's transaction commit boundary and invoke them from a child test process that terminates via `std::process::exit`. On pre-commit termination, a fresh process must observe neither the record nor the tip and must be able to commit the same height. On post-commit termination, a fresh process must observe the complete record and tip and an exact retry must be idempotent.
- **Isolation:** Hooks are private implementation details; no test hook is exposed by the public journal API. The code and tests remain in `experiments/malachite-spike`, outside the production Cargo workspace.
- **Evidence limit:** Abrupt userspace process exit is not a power cut, kernel crash, torn-write test, device failure, fsync audit or filesystem durability guarantee. The tests establish only the observed behavior on the CI runner and selected transaction boundaries.
- **Non-decisions:** No production durability claim, multi-process safety, backup rollback resistance, authenticated restore, cross-subsystem atomicity or Malachite runtime integration is approved.
- **Consequence:** The journal has a more meaningful process-restart test boundary, but G2/G4 remain blocked until full recovery and cross-subsystem consistency are designed, fault-tested and independently reviewed.
