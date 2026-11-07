# Persistent State Storage Architecture — Phase 9

**Status:** architecture specification; no durable backend is implemented.

## Scope

The current `apex-state` crate provides versioned snapshots, version-bound block plans, a `StateStore` contract, and a process-local `InMemoryStateStore`. This document defines requirements for a future durable adapter without binding protocol semantics to a database's physical format.

## Logical data model

The logical schema is deliberately expressed independently of database table APIs:

| Record | Logical key | Logical value / invariant |
|---|---|---|
| Store metadata | fixed metadata key | schema version, storage format version, initialization marker |
| State head | fixed head key | committed StateVersion and future committed block reference |
| Account state | canonical account key (not yet defined) | account nonce, balance, and future execution state |
| Commit journal | monotonically ordered commit identifier | parent version, target version, and recovery metadata if required by chosen backend |
| Migration marker | migration identifier | completed migration version; migration must be atomic or restartable |

This is not a serialization specification. No Rust struct layout, serde output, database key encoding, or current `AccountId(u64)` is approved as protocol bytes. The schema must evolve after the canonical account model and codec are ratified.

## Atomic block application

A production adapter's apply operation must behave as a compare-and-swap transaction:

1. Begin a serializable write transaction (or equivalent exclusive commit protocol).
2. Read the persisted head and require exact equality with the plan's parent StateVersion.
3. Validate every account before-state against the same transaction snapshot.
4. Validate account keys are unique and in the required deterministic order.
5. Write every after-state.
6. Write the incremented state version and any commit metadata in the same transaction.
7. Commit with explicitly selected durable semantics.
8. Return success only after the backend's documented durability boundary is satisfied.

Any precondition failure aborts without visible writes. An error returned during commit may be indeterminate: the caller must not blindly replay against a possibly committed head. Reopen the store, read the committed head, and reconcile by version and commit identity before continuing.

## Snapshot and concurrency contract

- A snapshot must pin or otherwise identify one consistent committed version.
- A snapshot cannot mix account reads from different commits.
- Concurrent readers may continue against older snapshots only while the backend guarantees their consistency.
- Exactly one writer may successfully advance a given parent version.
- The version and account records must be published atomically.
- StateVersion remains a local concurrency token. A future block-height and block-hash binding is separate protocol work.

## Recovery and operational requirements

- On open, validate metadata and supported schema versions.
- Reject unknown future schema versions; do not downgrade or reinterpret.
- Run only documented backend recovery procedures.
- Verify the state head and required commit metadata are internally consistent.
- Surface corruption and incomplete migrations as explicit errors.
- Provide a consistent backup/restore procedure; copying a live database file without its required journal/WAL companions is prohibited.
- Test disk-full, short-write, I/O error, process termination, OS interruption where test infrastructure permits, and reopen after each failure point.
- No recovery path may synthesize account values or advance the head without a complete committed block application.

## Storage-engine evaluation

### redb

Leading candidate for an embedded Rust prototype. Its published documentation describes ACID transactions, MVCC, concurrent readers, crash-safe behavior, and a stable file format. Version 4.5.0 was released on 2026-11-05 and adds multi-process concurrency modes. Because this release is very recent, pinning it now would be premature. Review the exact release's MSRV, license, upgrade notes, locking modes, durability API, and failure semantics before a spike. Evaluate immediate durability and two-phase commit settings against APEX's threat model; do not confuse its internal non-cryptographic checksums with APEX state commitments.

### SQLite

Mature alternative with documented serializable ACID transactions and WAL-mode concurrent readers/writer behavior. WAL introduces checkpointing and requires keeping the database, WAL, and shared-memory artifacts managed correctly. Use a maintained SQLite version at least 3.51.3 if WAL mode is selected, because the SQLite project documents a WAL-reset bug fixed in 3.51.3. Configure synchronous/durability behavior explicitly; a faster setting that can lose committed transactions after power failure is not acceptable for finalized state.

### RocksDB

Not selected for the first spike. The current Rust wrapper documentation declares Rust 1.88 MSRV and requires Clang/LLVM, while this workspace currently declares Rust 1.85. Revisit only after an explicit workspace MSRV decision and operational benchmark evidence.

### Excluded from immediate adoption

No backend is approved solely because it is fast, embedded, Rust-native, or has a key-value API. Sled is not selected without a fresh maintenance, release, compatibility, and recovery review.

## Backend conformance suite

Every candidate adapter must pass the same black-box suite:

1. Create, close, reopen, and verify an unchanged snapshot.
2. Apply a valid multi-account block; verify all writes and one version increment.
3. Attempt a stale plan; verify no mutation.
4. Attempt a plan with a late before-state mismatch; verify no partial write.
5. Race two plans against the same parent version; exactly one may commit.
6. Hold a reader snapshot while a writer commits; verify the reader remains internally consistent.
7. Inject failure before commit, during commit, and after commit acknowledgement boundaries; reopen and verify a complete old or complete new state only.
8. Simulate disk-full and storage I/O errors; fail closed.
9. Corrupt metadata/account records in test fixtures; report corruption, never default.
10. Exercise schema migration interruption and restart.
11. Verify backup/restore reproduces the same logical state and version.
12. Run deterministic randomized operation sequences against the in-memory reference model.

## Non-goals

This phase does not implement a database adapter, state root, authenticated trie, block journal recovery algorithm, pruning, snapshots export format, state sync, or database benchmark. Those require separate implementation and review gates.
