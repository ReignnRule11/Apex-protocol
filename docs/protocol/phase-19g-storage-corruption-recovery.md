# Phase 19G — Persistent-state corruption and recovery validation

**Status:** Implementation submitted for CI and review.
**Release impact:** G1 and G4 remain BLOCKED; overall release remains NO-GO.

## Objective

Strengthen the Phase 19F persistent full-width EVM state adapter's reopen boundary. The adapter must reject damaged or unsupported metadata and malformed records rather than silently initializing a replacement database or interpreting invalid bytes as state.

## Implemented validation

- Reopen rejects metadata where only one of schema version/current state version exists.
- Reopen rejects unknown schema versions without overwriting the existing database.
- Reopen rejects malformed current-version account records.
- Tests create corruption through direct redb writes, close the database handle, and exercise the public reopen path.
- Existing valid-state initialization, full-width persistence, replay rejection, before-state mismatch and redb integrity tests remain in the workspace suite.

## Recovery semantics

- A database that fails metadata or record validation is not automatically repaired.
- The adapter does not fall back to the supplied initial snapshot when existing metadata is incomplete or unsupported.
- Operators must preserve the original database and recover from a separately verified backup or a future, explicitly specified recovery procedure.
- redb integrity verification checks storage-engine consistency only; it does not establish protocol-state correctness or a cryptographic state root.

## Explicit limitations

- These are deterministic corruption-injection tests, not actual OS crash, power-loss, disk-full, torn-write or hardware-failure simulations.
- No commit-acknowledgement fault injection is claimed in this phase.
- No automatic backup/restore, schema migration, orphaned-version reconciliation, version garbage collection, multi-process writer test or consensus/signer atomic commit.
- The state database still lacks canonical serialization and a complete protocol state commitment.
- The EVM execution result is not yet automatically converted into EvmStateDelta.

## Validation

PR and CI evidence will be added after the implementation branch passes Format, Clippy and the full workspace tests.

## Release decision

**NO-GO.** This phase improves fail-closed reopen validation; it does not satisfy G4 durable operation or authorize production use.
