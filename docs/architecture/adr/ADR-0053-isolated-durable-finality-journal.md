# ADR-0053 — isolated durable finality journal prototype

- **Status:** Accepted for isolated research only; not production adoption.
- **Date:** 2026-11-07
- **Context:** Phase 20G2C5 defines idempotent commit outcomes but has no concrete persistence implementation. A finality callback must be able to distinguish an exact retry from a conflicting identity and recover the committed payload after restart.
- **Decision:** Add an isolated redb 4.5.0 prototype under `experiments/malachite-spike`. It writes the identity/payload journal row and chain-tip row in one redb write transaction. A commit error is ambiguous. Same identity and same payload is idempotent; a different identity at the same height, or a different payload for the same identity, fails closed. New heights must be contiguous and parent-linked.
- **Isolation:** The experiment remains outside the production Cargo workspace. It does not change the production storage dependency graph.
- **Non-decisions:** No claim of cross-database atomicity, power-loss durability, multi-process safety, backup rollback resistance, authenticated state sync, or production readiness. No live Malachite runtime or signer integration is included.
- **Consequence:** A concrete persistence behavior can now be compiled and tested independently, while the production gate remains closed until fault-injection and integrated recovery evidence exists.
