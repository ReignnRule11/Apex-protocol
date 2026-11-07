# Phase 19D — revm database trait adapter

**Status:** Merged; PR #24 and post-merge CI passed.
**Release impact:** G1 remains BLOCKED.

## Objective

Connect the immutable Phase 19C EVM state view to the exact pinned revm 43.0.3 Database trait. The adapter preserves full-width values, keeps reads bound to one parent snapshot, and fails closed when code is absent or malformed.

## Implementation

- Implements revm's Database trait directly for the APEX EvmDatabaseView.
- Converts 20-byte addresses, 256-bit balances, storage keys and storage values without narrowing.
- Returns account information with code loaded lazily by its code hash.
- Converts raw bytecode through revm's checked bytecode constructor.
- Returns a fatal typed error for missing code hashes and malformed bytecode; it never substitutes empty code.
- Reads block hashes from the same snapshot and returns zero for unavailable entries.
- Keeps the adapter read-only; revm state changes are not committed to APEX state by this phase.

## Validation

PR #24: https://github.com/ReignnRule11/Apex-protocol/pull/24
PR CI: https://github.com/ReignnRule11/Apex-protocol/actions/runs/42386866316
Post-merge CI: https://github.com/ReignnRule11/Apex-protocol/actions/runs/42386927064

Both checks completed successfully.

## Trust and correctness boundaries

The adapter trusts the snapshot's code-hash-to-bytecode association. It does not recompute the hash, establish canonical code storage, enforce the BLOCKHASH historical window, or validate that a block context is fork-correct. Those rules require a ratified fork and protocol-level state validation.

The adapter uses the immutable Phase 19C snapshot; it does not touch the legacy AccountId(u64)/u128 redb store.

## Next phase

Phase 19E defines ordered before/after account, code and storage updates and an all-or-nothing in-memory application boundary. See phase-19e-atomic-evm-state-delta.md and ADR-0029.

**G1 remains BLOCKED. Release decision: NO-GO.**
