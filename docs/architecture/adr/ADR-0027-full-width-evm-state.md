# ADR-0027: Full-width EVM state and immutable read boundary

- **Status:** Accepted after PR #23 and post-merge CI passed
- **Date:** 2026-11-07
- **Related:** ADR-0024, ADR-0025, ADR-0026, Phase 19C

## Context

The historical APEX state prototypes use AccountId(u64), u64 nonce and u128 balance. They were useful for admission and storage lifecycle experiments but cannot represent the protocol draft's EVM account address and 256-bit balance/storage domains. Reusing them would silently narrow EVM state.

## Decision

1. Introduce EVM-native fixed-width types: 20-byte addresses and 32-byte unsigned words.
2. Keep account, code, storage and block-hash domains separate.
3. Construct immutable snapshots bound to one internal parent-state version.
4. Require explicit caller-supplied resource limits and reject over-limit snapshots.
5. Validate account-to-code and storage-to-account references before exposing a read view.
6. Keep the read view independent of revm types. Implement the revm Database conversion in a follow-on phase after its exact API and fork semantics are reviewed.
7. Do not claim code-hash verification, canonical encoding, persistent storage, state-root completeness or execution readiness.

## Consequences

- EVM state no longer depends on the legacy narrow account identifier model.
- APEX can test full-width state lookup and snapshot isolation independently from engine-specific types.
- The current redb adapter remains a legacy prototype and is not the backend for this model.
- G1 remains blocked; release remains NO-GO.
