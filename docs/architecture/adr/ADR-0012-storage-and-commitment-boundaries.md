# ADR-0012: Persistent Storage and State Commitment Boundaries

- **Status:** Accepted architecture gate; backend and commitment algorithms remain undecided
- **Date:** 2026-11-07

## Context

Phase 8 provides versioned snapshots and an in-memory compare-and-swap adapter. A production state layer must persist account changes and the state version atomically, recover consistently after interruption, and expose stable snapshots. Separately, validators need a deterministic cryptographic commitment to application state.

These are related but distinct problems. A database's internal page checksums, B-tree roots, transaction IDs, or file layout are not APEX consensus commitments. A storage migration must not change a block's state root.

The canonical protocol encoding remains open (ADR-0005). AccountId is currently a u64 prototype identifier, not a protocol account address. It would be unsafe to hash arbitrary Rust or database serialization and call the result a protocol state root.

## Decision

1. Keep storage behind the existing `StateStore` abstraction. A production implementation must provide consistent snapshots and linearizable all-or-nothing block application.
2. Do not add a storage-engine dependency in this phase. redb is the leading embedded Rust candidate for a focused evaluation; SQLite is the mature alternative. Neither is approved for production.
3. Keep state commitment semantics independent of the storage engine and its physical layout.
4. Do not implement or publish a consensus state root until canonical account encoding, key ordering, hash function, domain separation, tree structure, empty-tree semantics, and versioning are specified and test-vectored.
5. Treat the Phase 9 storage schema and commitment documents as design contracts, not operational persistence or cryptographic implementation.

## Required production storage invariants

- A snapshot observes one committed state version only.
- A block application atomically writes all account changes and the new version, or none.
- The commit result is unambiguous; after an indeterminate I/O error, the adapter must reopen/reconcile before accepting another block.
- Durable commit semantics are explicit and use the backend's strongest required durability mode.
- Concurrent writers cannot both commit against the same parent version.
- Reopening after process, OS, or power interruption yields a valid committed state, never a mixture of pre- and post-block account writes.
- Corruption is reported; the adapter never silently substitutes defaults or repairs consensus state by guessing.
- Backups include all backend-required journal/WAL files or use a supported consistent backup mechanism.

## Consequences

No durable adapter or state root is implemented by ADR acceptance. The next implementation gate is a backend spike and conformance suite. Backend selection requires exact version/MSRV/license review, dependency audit, fault-injection evidence, reopen/recovery tests, and performance measurements on representative APEX workloads.

A future state-root ADR must be independently reviewable and include cross-implementation vectors. Changing the database must not alter state-root bytes for identical logical state.
