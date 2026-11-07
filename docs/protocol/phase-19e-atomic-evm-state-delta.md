# Phase 19E — Full-width EVM state deltas and atomic application

**Status:** Implementation submitted for CI validation.
**Release impact:** G1 remains BLOCKED.

## Objective

Define an APEX-owned representation for account, code and storage changes produced by EVM execution. Apply a delta only to the exact parent snapshot from which it was derived, and publish the new snapshot only if every precondition and resulting-state invariant passes.

## Implementation

- EvmAccountUpdate, EvmCodeUpdate and EvmStorageUpdate carry before/after values.
- EvmStateDelta binds the complete update set to one parent state version.
- Each update list must be strictly increasing by its full key; duplicates and unordered entries are rejected.
- EvmStateStore checks parent version and every before-state before constructing a replacement.
- Updates are applied to cloned full-width maps; the resulting snapshot is validated against account/code, storage/account and caller-supplied resource-limit invariants.
- The live store is replaced only after the entire new snapshot validates.
- Failed, stale, replayed, unordered or invalid-reference deltas leave the original snapshot unchanged.
- An empty delta advances the internal state version once, allowing an empty block transition to have a distinct parent/child version.

## Validation

Tests cover full-width account and storage changes, stale/replayed deltas, a late storage mismatch after an earlier valid account update, removal of code still referenced by an account, duplicate updates and version overflow.

## Deliberate boundaries

This is a process-local in-memory reference store only. It is not persistent, concurrent, crash-safe or connected to the legacy redb adapter. It does not implement revm DatabaseCommit, transaction execution state-diff extraction, canonical delta encoding, complete state roots, authenticated transactions, fee settlement, receipts or fork activation.

The code-hash association is still referential rather than cryptographically recomputed. State-diff ordering is deterministic for the in-memory representation but not yet a canonical wire format.

## Next work

- Define a persistent full-width state schema and atomic commit/recovery semantics.
- Prove crash consistency, migration and snapshot/restore behavior.
- Connect actual revm execution result changes to EvmStateDelta without bypassing validation.
- Add official fork-specific execution fixtures, differential testing and fuzzing.

**G1 remains BLOCKED. Release decision: NO-GO.**
