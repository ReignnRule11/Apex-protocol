# Phase 19F — Persistent full-width EVM state prototype

**Status:** Merged — PR #26; PR CI and post-merge CI passed.
**Release impact:** G1 and G4 remain BLOCKED.

## Objective

Persist the Phase 19C full-width EVM state without narrowing values or reusing the historical AccountId(u64)/u128 redb schema.

## Storage model

New workspace crate: `crates/apex-evm-redb`.

Tables:
- `apex_evm_metadata_v1`: schema version and current internal state version.
- `apex_evm_accounts_v1`: version + 20-byte address key; nonce + 32-byte balance + 32-byte code-hash value.
- `apex_evm_code_v1`: version + 32-byte opaque code hash key; raw bytecode value.
- `apex_evm_storage_v1`: version + 20-byte address + 32-byte slot key; 32-byte value.
- `apex_evm_block_hashes_v1`: version + block-number key; 32-byte hash value.

All key and fixed-width field bytes are big-endian. This is a private database layout, not a canonical wire encoding or state commitment.

## Atomic publication

Each new snapshot is stored under a fresh version prefix. A single redb write transaction inserts all account, code, storage and block-hash records, then updates the metadata current-version pointer. Existing versions are immutable and retained. Readers load the version named by metadata and validate the rebuilt snapshot through EvmStateSnapshot::try_new.

The in-memory candidate is published only after redb acknowledges commit. If commit returns an error, the adapter is latched as ReopenRequired; callers must close and reopen to reconcile the durable pointer before retrying.

## Tests

- Initialize and reopen all four state domains with full-width values.
- Apply and persist a successful non-empty account/code/storage delta; verify all changed domains after reopen.
- Apply an empty new version; reject replay both before and after reopen.
- Reject a failed before-state check without advancing durable version.
- Prove an existing database is not overwritten by a new initial snapshot.
- Run redb integrity verification after a committed snapshot.

Validation links: [PR #26](https://github.com/ReignnRule11/Apex-protocol/pull/26), [PR CI #42388264492](https://github.com/ReignnRule11/Apex-protocol/actions/runs/42388264492), [post-merge CI #42388315877](https://github.com/ReignnRule11/Apex-protocol/actions/runs/42388315877).

## Explicit limitations

- Single process/single writer under redb's default exclusive file lock.
- Historical snapshots are append-only; no garbage collection or compaction.
- Caller-supplied resource limits are not consensus parameters.
- No online backup, multi-process race testing, power-loss simulation or independent audit.
- No canonical encoding, complete state root, state sync, schema migration or consensus/signer atomic commit.
- The EVM execution result is not yet converted automatically into EvmStateDelta.

## Release status

This is a persistence prototype, not a production storage guarantee. G1 and G4 remain blocked; overall release remains NO-GO.
