# ADR-0029: Atomic full-width EVM state delta application

- **Status:** Proposed pending CI validation
- **Date:** 2026-11-07
- **Related:** ADR-0027, ADR-0028, Phase 19E

## Context

The revm Database adapter is read-only and bound to an immutable full-width snapshot. Execution must eventually produce changes to account metadata, bytecode and storage. Publishing any subset of those changes would leave an internally inconsistent EVM state.

## Decision

1. Represent account, code and storage changes as explicit before/after updates.
2. Bind every delta to the exact parent snapshot version.
3. Require each update list to be strictly ordered and unique by its full key.
4. Check all before-state values before constructing the new state.
5. Apply changes to private copies and validate the complete resulting snapshot, including resource bounds and account/code/storage references.
6. Publish the replacement snapshot only after all checks succeed.
7. Keep this phase process-local and in-memory. Persistence, concurrency, crash recovery and revm DatabaseCommit remain separate work.
8. Permit an empty delta to advance the internal state version, representing a block with no state changes.

## Consequences

- Stale and replayed deltas cannot overwrite a newer snapshot.
- A late invalid update cannot partially publish earlier valid updates.
- Code cannot be removed while a resulting account still references it; storage cannot remain attached to an absent account.
- The legacy AccountId(u64)/u128 state and redb adapter remain isolated.
- G1 remains blocked until persistent state, execution integration, fees, receipts, commitments and conformance evidence exist.

## Rejected alternatives

- Mutating account, code or storage maps incrementally before all checks pass.
- Accepting duplicate or nondeterministically ordered updates.
- Applying a delta to whichever snapshot happens to be current.
- Claiming crash safety from an in-memory atomic replacement.
