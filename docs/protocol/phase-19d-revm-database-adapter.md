# Phase 19D — revm database trait adapter

**Status:** Implementation submitted for CI validation.
**Release impact:** G1 remains BLOCKED.

## Objective

Connect the immutable Phase 19C EVM state view to the exact pinned revm 43.0.3 Database trait. The adapter must preserve full-width values, keep reads bound to one parent snapshot, and fail closed when code is absent or malformed.

## Implementation

- Implements revm's Database trait directly for the APEX EvmDatabaseView.
- Converts 20-byte addresses, 256-bit balances, storage keys and storage values without narrowing.
- Returns account information with code loaded lazily by its code hash.
- Converts raw bytecode through revm's checked bytecode constructor.
- Returns a fatal typed error for missing code hashes and malformed bytecode; it never substitutes empty code.
- Reads block hashes from the same snapshot and returns zero for unavailable entries.
- Keeps the adapter read-only; revm state changes are not committed to APEX state by this phase.
- Tests the trait-facing conversion, missing account behavior, storage, block hashes and malformed/missing code.

## Trust and correctness boundaries

The adapter trusts the snapshot's code-hash-to-bytecode association. It does not recompute the hash, establish canonical code storage, enforce the BLOCKHASH historical window, or validate that a block context is fork-correct. Those rules require a ratified fork and protocol-level state validation.

The adapter uses the immutable Phase 19C snapshot; it does not touch the legacy AccountId(u64)/u128 redb store.

## Deliberate non-goals

- Mutable revm database commit or atomic state-diff persistence.
- Full-width persistent database schema or migration.
- Canonical account/storage serialization or complete state commitment.
- Transaction authentication, nonce admission, EIP-1559 fees, receipts or logs.
- Ratified EVM fork, execution context, official fixture suite or differential client tests.
- Production readiness or security audit.

## Validation and next work

CI must pass Format, Clippy and workspace tests. The next phase should define a full-width state-diff representation and atomic application boundary, then add explicit execution context and fork-specific conformance tests.

**G1 remains BLOCKED. Release decision: NO-GO.**
