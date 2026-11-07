# Phase 19C — Full-width EVM state model and immutable read adapter

**Status:** Implementation submitted for CI validation.
**Release impact:** G1 remains BLOCKED; production execution is not enabled.

## Objective

Introduce an EVM-native state representation without reusing the historical AccountId(u64) / u128 transfer prototype. Provide a bounded, immutable read view tied to one parent-state version so an eventual revm adapter cannot mix values from different state snapshots.

## Implemented in this phase

- EvmAddress([u8; 20]) for full-width EVM addresses.
- U256Word([u8; 32]) for balances and storage keys/values, represented big-endian in memory.
- EvmCodeHash([u8; 32]), account nonce, full-width balance and code reference.
- Separate maps for present accounts, bytecode, contract storage and historical block hashes.
- EvmStateSnapshot::try_new validates configured count/byte limits, code-reference presence and storage ownership before publishing a snapshot.
- EvmDatabaseView exposes immutable basic, code_by_hash, storage and block_hash reads tied to the snapshot's parent state version.
- Missing storage reads return zero; absent accounts remain distinguishable from present empty accounts.
- Resource limits are caller-supplied. No consensus limits or fork-dependent pruning rules are silently selected.
- Tests cover full-width values, distinct 20-byte addresses, absent accounts/slots, version binding, dangling references and resource-limit rejection.

## Deliberate boundaries

This phase does not implement or claim:
- revm::Database trait conversion; the APEX read model remains engine-independent until the exact conversion contract is reviewed.
- A persistent full-width state backend, atomic write adapter or transaction state-diff commit.
- Code-hash recomputation/validation, canonical account/storage encoding or a complete state commitment.
- Authenticated transaction decoding, sender recovery, nonce/fee admission or EIP-1559 settlement.
- A ratified EVM fork, explicit consensus-derived block environment, precompiles or official execution fixtures.
- Cross-client, cross-architecture or production determinism.

The current redb adapter persists only the legacy AccountId(u64) / u128 prototype. It must not be presented as backing this EVM model.

## Validation

CI must pass formatting, Clippy and all workspace tests. The next phase should add the revm database-trait conversion against this immutable view, with explicit conversion tests and no state mutation through the read interface.

## Exit criteria for G1

The full application path still requires authenticated canonical transactions, a ratified fork and execution context, deterministic execution, actual gas/fee settlement, receipts/logs, complete post-state commitment, official fork-specific fixtures, differential testing, fuzzing, bounded resource use and independent security review.

**Release decision: NO-GO.**
