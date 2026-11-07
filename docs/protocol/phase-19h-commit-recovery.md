# Phase 19H — Commit ambiguity and orphaned-version recovery tests

**Status:** Implementation submitted for CI and review.
**Release impact:** G1 and G4 remain BLOCKED; overall release remains NO-GO.

## Objective

Exercise the persistent full-width EVM adapter's commit/reopen contract at the point where a caller can no longer infer whether a durable commit happened. Verify that only the version referenced by committed metadata is selected after reopen.

## Implemented test boundaries

- **Pre-commit failure:** a test-only fault drops the uncommitted redb write transaction, latches the adapter, and verifies reopen retains the parent version. A retry after reopen can then apply the original delta.
- **Lost acknowledgement:** a test-only fault commits successfully and then returns a simulated acknowledgement error. The adapter latches; reopen discovers the committed next version; replaying the old delta is rejected as stale.
- **Orphaned future-version records:** test-only setup writes a complete future snapshot without advancing the metadata current-version pointer. Reopen must continue to select the pointer-referenced parent version. A subsequent valid transition can publish that next version.

## Safety properties exercised

1. In-memory state is not exposed after an ambiguous persistence outcome; the instance requires close/reopen.
2. The durable metadata pointer, not the presence of version-prefixed records alone, determines the active snapshot.
3. An acknowledged or ambiguously acknowledged commit is reconciled by reopening before retry.
4. A stale transaction cannot be replayed after a commit that actually succeeded.

## Explicit limitations

- Fault injection is compiled only for tests and does not add production controls or public API surface.
- The acknowledgement-loss test commits through redb and simulates only the lost response; it does not reproduce a real process/network/device fault.
- The pre-commit test drops a transaction before commit; it is not a disk-full or torn-write simulation.
- No OS crash, power-loss, filesystem fault, multiprocess writer, backup/restore, version garbage collection or consensus/signer atomic commit is established.
- The adapter still uses a single-process/single-writer model and has no canonical state root.

## Validation

PR and CI evidence will be recorded after Format, Clippy and the full workspace test suite complete.

## Production decision

**NO-GO.** This phase tests selected reconciliation semantics; it does not satisfy G4 durable operation or authorize production use.
