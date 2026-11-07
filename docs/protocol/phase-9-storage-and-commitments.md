# Phase 9 — Storage and State Commitment Architecture

**Status:** design gate completed; implementation gate remains open.

## Objective

Define a backend-independent persistence contract and a safe path to deterministic state commitments without prematurely binding consensus to a database format or non-canonical serialization.

## Baseline verified

Phase 8 added versioned snapshots, version-bound transition plans, and an in-memory compare-and-swap adapter. It is not durable, crash-safe, or multi-process. The canonical codec remains intentionally unconfigured.

## Decisions recorded

- Storage and consensus commitment semantics are independent.
- No database dependency is added in Phase 9.
- redb is the leading embedded Rust candidate for a focused spike, but its 4.5.0 release is too recent to adopt without exact-version validation.
- SQLite WAL is the mature alternative, subject to explicit durability/checkpoint policy and a fixed minimum SQLite version.
- RocksDB is deferred because the current Rust wrapper's declared MSRV is above the workspace MSRV.
- No state-root algorithm or hash suite is selected.
- No durable state adapter or Merkle implementation is claimed.

## Deliverables

- ADR-0012 records the storage/commitment boundary and decision gates.
- `storage-architecture.md` specifies logical records, atomic application, snapshot isolation, recovery and backend conformance.
- `state-commitments.md` specifies state-root requirements and candidate tree structures.
- This document records scope and remaining gates.

## Acceptance criteria for Phase 9 design

- [x] Separate database integrity from consensus state commitments.
- [x] Define atomic version + account update semantics.
- [x] Define consistent snapshot and concurrent-writer requirements.
- [x] Define crash/reopen and corruption behavior.
- [x] Compare storage candidates against current MSRV and operational constraints.
- [x] Preserve fail-closed canonical codec boundary.
- [x] Update repository documentation and tree.
- [ ] Select and pin a storage backend after a dedicated spike.
- [ ] Implement and pass backend conformance/fault-injection suite.
- [ ] Ratify canonical state encoding and authenticated structure in a separate ADR.
- [ ] Implement cross-language commitment test vectors and independent review.

## Next phase

Phase 10 should be a contained storage-engine spike. Start with a pinned redb candidate only after confirming its exact Rust 1.85 compatibility; if incompatible, evaluate SQLite with an explicit maintained SQLite version and durability configuration. Implement the adapter behind `StateStore`, then run the shared conformance suite. Do not implement a state root in that phase unless the codec and commitment decisions are separately ratified.
