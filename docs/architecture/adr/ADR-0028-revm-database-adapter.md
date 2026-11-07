# ADR-0028: revm database adapter over immutable EVM snapshots

- **Status:** Accepted after PR #24 and post-merge CI passed
- **Date:** 2026-11-07
- **Related:** ADR-0024 through ADR-0027, Phase 19D

## Context

Phase 19C established a full-width, immutable EVM state view. The pinned revm 43.0.3 API exposes a Database trait for account information, bytecode, storage and historical block hashes. The adapter must not bypass the snapshot boundary or reuse the legacy narrow state store.

## Decision

1. Implement revm 43.0.3 Database directly for EvmDatabaseView.
2. Convert 20-byte addresses and 256-bit balances/storage keys/values losslessly.
3. Return AccountInfo with bytecode fetched lazily through code_by_hash.
4. Parse bytecode using revm's checked constructor and surface typed fatal errors for missing or malformed code.
5. Resolve block-hash reads from the same immutable snapshot; unavailable values return zero.
6. Keep the adapter read-only. No DatabaseCommit implementation is provided.
7. Treat code-hash correctness, fork-dependent BLOCKHASH rules, persistent storage, state diffs and state commitments as unresolved protocol work.

## Consequences

- The EVM engine can consume an APEX-owned immutable snapshot through its native database interface.
- Engine reads cannot mutate the parent snapshot.
- The adapter remains dependent on snapshot validation and the eventual fork/context contract.
- G1 remains blocked until transaction authentication, state-diff application, fee settlement, receipts, complete commitments and conformance evidence exist.
