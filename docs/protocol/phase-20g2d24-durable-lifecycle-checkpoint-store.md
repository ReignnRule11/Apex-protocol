# Phase 20G2D24 — Durable Lifecycle Checkpoint Store and Rollback Witness

## Objective

Replace the test-only in-memory checkpoint persistence with a durable transactional adapter and add an explicit startup check against an independently protected rollback witness.

## Implementation

- Add `apex-lifecycle-redb`, a dedicated redb adapter for `MonotonicLifecycleCheckpointStore`.
- Persist network-scoped sequence/hash heads in one redb write transaction.
- Use exact compare-and-advance semantics; stale writers cannot overwrite a newer head.
- Validate schema metadata and fixed-width records; fail closed on invalid stored values.
- Add explicit redb integrity verification.
- Add `ExternalLifecycleRollbackWitness` and `verify_external_rollback_witness` to compare local state with an independent trust domain.
- Test close/reopen persistence, stale writer rejection, invalid sequence rejection, integrity verification and rollback-witness mismatch detection.

## Security boundary

The redb adapter provides durable local persistence and atomic single-database compare-and-advance. redb's exclusive file lock makes this implementation single-process/single-handle; it is not a distributed multi-writer database.

**This does not by itself provide anti-rollback.** Restoring an older database image, replacing the storage device, or restoring both database and backups can revert the local head. The witness must be independently protected and monotonic (for example, a separately administered remote service or hardware-backed trust domain). The included witness is an interface and in-memory test double only; no production witness provider, TPM/HSM integration, backup protocol, or disaster-recovery procedure is implemented.

A mismatch is fail-closed. Automatic reconciliation is intentionally omitted because choosing whether local or witness state is authoritative requires a ratified recovery policy.

## Validation

All repository CI jobs must pass on the exact PR head. No merge is performed by this phase.

## Release posture

This is a durable storage adapter and rollback-detection integration boundary, not a production anti-rollback deployment. G0–G5 remain blocked; G6/G7 remain not started; overall release remains **NO-GO**.
