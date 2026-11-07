# ADR-0044: Isolated Multi-Process Adversarial Harness

**Status:** Accepted for test-harness prototype only; BFT integration and production validator operation NOT approved.
**Date:** 2026-11-07

## Context

Phases 20C–20E established an engine-neutral application boundary, a durable signing-reservation prototype and deterministic validator-set primitives. None is an integrated BFT engine or a networked validator implementation. Before engine integration, APEX needs reproducible process-lifecycle and message-delivery fault fixtures without implying consensus correctness.

## Decision

Create a separate Cargo workspace at `experiments/consensus-harness`, excluded from the production workspace. Its coordinator launches four independent OS worker processes. Each worker maintains a separate append-only test journal and accepts bounded commands over stdin/stdout.

The test suite covers ordered replication, partition/gap behavior, delayed/dropped/reordered delivery, conflicting record rejection, process restart and journal reload, replacement-process replay, byte-identical deterministic replay, and per-message payload limits. Successful appends are synced before acknowledgment. The CI workflow runs format, Clippy and integration tests independently from the production workspace and Malachite spike.

## Deliberate non-decisions

This does not implement or select Malachite, BFT consensus, vote authentication, signature/certificate verification, quorum agreement, peer discovery, network sockets, Byzantine cryptographic identities, production WAL, state sync, validator economics or finality. The test coordinator simulates delivery schedules; it does not reproduce real network timing or prove safety/liveness.

## Consequences and residual risks

The fixture creates an executable baseline for process isolation, deterministic fault schedules and restart/replay testing. Its scope is intentionally narrower than the full Phase 20F adversarial integration goal. G2 and G3 remain blocked until an engine-backed, authenticated, networked multi-validator test suite is implemented and independently reviewed.

## References

- Phase 20C: `docs/protocol/phase-20c-application-engine-boundary.md`
- Phase 20D: `docs/protocol/phase-20d-durable-signer-safety.md`
- Phase 20E: `docs/protocol/phase-20e-validator-lifecycle.md`
- Phase 20F: `docs/protocol/phase-20f-multiprocess-adversarial-harness.md`
