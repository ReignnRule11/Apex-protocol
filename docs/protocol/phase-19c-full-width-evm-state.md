# Phase 19C — Full-width EVM state model and immutable read adapter

**Status:** Merged; PR #23 and post-merge CI passed.
**Release impact:** G1 remains BLOCKED.

## Objective

Introduce an EVM-native state representation without reusing the historical AccountId(u64) / u128 transfer prototype. Provide a bounded, immutable read view tied to one parent-state version so an eventual revm adapter cannot mix values from different state snapshots.

## Implemented

- EvmAddress([u8; 20]) and U256Word([u8; 32]) for full-width address, balance and storage representation.
- Separate account, bytecode, storage and block-hash maps.
- Immutable EvmStateSnapshot bound to one internal parent-state version.
- Explicit caller-supplied account/code/storage/block-hash limits.
- Validation of account-to-code and storage-to-account references.
- Read-only EvmDatabaseView for basic, code, storage and block-hash lookups.
- Tests for full-width values, absent accounts/slots, version binding, dangling references and resource-limit rejection.

## Validation

PR #23: https://github.com/ReignnRule11/Apex-protocol/pull/23
PR CI: https://github.com/ReignnRule11/Apex-protocol/actions/runs/42386406530
Post-merge CI: https://github.com/ReignnRule11/Apex-protocol/actions/runs/42386457898

Both checks completed successfully.

## Scope limits

Phase 19C does not implement revm's Database trait, persistent full-width state, atomic state-diff application, code-hash verification, canonical state encoding or a complete state commitment. The existing redb adapter persists only the legacy AccountId(u64)/u128 prototype.

Phase 19D adds the read-only revm Database adapter over this snapshot. See phase-19d-revm-database-adapter.md and ADR-0028.

**Release decision: NO-GO.**
