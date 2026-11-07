# ADR-0030: Versioned redb persistence for full-width EVM state

- **Status:** Proposed pending CI validation
- **Date:** 2026-11-07
- **Related:** ADR-0027 through ADR-0029, Phase 19F

## Context

The Phase 19C–19E state model uses 20-byte addresses and 256-bit balances/storage values. The existing redb adapter persists only the historical AccountId(u64)/u128 prototype and cannot safely represent the EVM state. Reusing that schema would narrow values and conflate separate state models.

## Decision

1. Add a separate `apex-evm-redb` crate; do not modify or repurpose `apex-state-redb`.
2. Store accounts, code blobs, storage slots and block-hash entries in distinct redb tables.
3. Use fixed-width big-endian private storage keys and account/storage value layouts. These are database records, not canonical protocol serialization.
4. Prefix every record key with an internal state version. Preserve previous versions instead of in-place overwrites.
5. In one redb write transaction, write all records for the next version and advance the metadata current-version pointer.
6. On open, load only records matching the metadata version and rebuild an immutable EVM snapshot through the existing invariant validator.
7. Use redb's default exclusive file lock and Immediate durability. Latch the store after a commit error and require close/reopen to reconcile whether the metadata pointer advanced.
8. Keep resource limits caller-supplied; do not silently turn them into consensus constants.
9. Defer garbage collection, online backup, multi-process writers, cross-subsystem commit coordination and production database selection.

## Consequences

- Full-width EVM values are persisted without narrowing to the legacy state model.
- A crash before transaction commit should leave the prior metadata pointer and snapshot visible; a committed transaction publishes all new records with the pointer.
- Historical versions are retained, increasing disk usage until a separately designed compaction/retention policy exists.
- The state-version pointer is still an internal storage concurrency token, not block height or state root.
- This prototype does not prove power-loss behavior or production durability; fault-injection and recovery evidence remain required.
- G1 and G4 remain blocked.
