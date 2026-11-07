# Phase 19G — Persistent-state corruption and recovery validation

**Status:** Merged.
**PR:** #28 — https://github.com/ReignnRule11/Apex-protocol/pull/28
**Merge commit:** `c653b073e446620b5f6ede38c7d41c7bde2c72dd`
**Validation:** PR CI #42389694673 and post-merge main CI #42389851008 passed Format, Clippy and full workspace tests.
**Release impact:** G1 and G4 remain BLOCKED; overall release remains NO-GO.

## Objective

Strengthen the Phase 19F persistent full-width EVM state adapter's reopen boundary. The adapter must reject damaged or unsupported metadata and malformed records rather than silently initializing a replacement database or interpreting invalid bytes as state.

## Implemented validation

- Reopen rejects metadata where only one of schema version/current state version exists.
- Reopen rejects unknown schema versions without overwriting the existing snapshot.
- Reopen rejects malformed current-version account records.
- Tests inject controlled corruption through redb writes, close the database handle, and exercise the public reopen path.

## Explicit limitations

- These are deterministic corruption-injection tests, not actual OS crash, power-loss, disk-full, torn-write or hardware-failure simulations.
- No automatic repair, migration, backup/restore, orphaned-version reconciliation, version garbage collection, multi-process writer test or consensus/signer atomic commit.
- The state database still lacks canonical serialization and a complete protocol state commitment.
- The EVM execution result is not yet automatically converted into EvmStateDelta.

## Production decision

**NO-GO.** This phase improves fail-closed reopen validation; it does not satisfy G4 durable operation or authorize production use.
