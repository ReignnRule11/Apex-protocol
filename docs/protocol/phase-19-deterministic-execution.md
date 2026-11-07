# Phase 19 — Deterministic EVM Execution and Fee Accounting

**Status:** revm 43.0.3 candidate pinned; execution harness, full-width state model, read adapter, atomic in-memory deltas and a durable redb prototype are merged.
**Release impact:** G1 remains BLOCKED.

## Objective

Implement a deterministic EVM execution subsystem for the proposed initial EIP-1559 type-2 transaction profile against a complete versioned parent-state view. Execution must produce canonical receipts, gas accounting, fee settlement and a complete post-state commitment. A library call that executes bytecode is not, by itself, a blockchain execution layer.

## Engine candidate

- Engine: revm 43.0.3, exact version pin in crates/apex-evm/Cargo.toml.
- Toolchain: Rust 1.91.0, exact workspace and CI pin.
- Features: default features disabled; std enabled explicitly.
- Upstream license: MIT; APEX still requires a complete dependency/license inventory and security review.
- Fork: not ratified. Osaka/Fusaka-era behavior remains a candidate baseline; Glamsterdam remains excluded until finalized and explicitly selected.
- Adapter: APEX-owned boundary; engine types are not canonical wire/storage formats.

## Progress

### 19A — dependency and toolchain compatibility
PR #21 and post-merge CI passed for the exact revm 43.0.3 candidate under Rust 1.91.0.

### 19B — minimal execution API harness
PR #22 and post-merge CI passed. A test-only fixture deploys a contract, commits creation, calls it and checks a known storage value.

### 19C — full-width state model and immutable read boundary
PR #23 and post-merge CI passed. Adds 20-byte addresses, 256-bit balances/storage words, bytecode references and caller-bounded immutable snapshots.

### 19D — revm Database trait adapter
PR #24 and post-merge CI passed. Implements revm Database over the immutable snapshot, with lossless full-width conversions and typed fail-closed errors for missing/malformed code. It is read-only.

### 19E — atomic full-width EVM state deltas
PR #25 and post-merge CI passed. Ordered before/after account, code and storage updates are bound to a parent version. A process-local in-memory store validates every precondition and the complete resulting snapshot before publishing a replacement.

### 19F — persistent full-width EVM state prototype (merged)
PR #26 and post-merge CI passed. Adds `apex-evm-redb`, separate from the legacy `apex-state-redb` adapter. Account, code, storage and block-hash records use fixed-width big-endian keys/fields and a private storage schema. Each state version is written under version-prefixed keys. A single redb write transaction inserts the complete next snapshot and updates the metadata current-version pointer. Historical versions are retained, making publication atomic without mutating old records.

The adapter uses redb's default exclusive file lock and Immediate durability. It is single-process/single-writer. Resource limits are caller supplied and not protocol parameters. It has no garbage collection, online backup, multi-process coordination, canonical wire encoding, state root or atomic consensus/signer commit. Commit errors latch the instance as requiring reopen/reconciliation.

## Phase 19G — persistent-state corruption and recovery validation (merged)
PR #28 merged at `c653b073e446620b5f6ede38c7d41c7bde2c72dd`. PR CI #42389694673 and post-merge main CI #42389851008 passed Format, Clippy and the full workspace tests. It adds controlled reopen-corruption tests for incomplete metadata, unsupported schema versions and malformed current-version account records. These are not actual crash/power-loss or I/O fault simulations. See `phase-19g-storage-corruption-recovery.md` and ADR-0031.

## Phase 19H — commit ambiguity and orphaned-version recovery tests
Adds test-only fault injection at the durable commit boundary. It covers a pre-commit failure (transaction dropped, old version retained), a committed transaction whose acknowledgement is simulated as lost (store latches; reopen discovers the advanced version; replay is rejected), and orphaned future-version records (not selected without the metadata pointer). These tests validate the adapter's reconciliation contract; they do not simulate OS/device faults or prove redb's power-loss behavior. See `phase-19h-commit-recovery.md` and ADR-0032.

## Required next work

1. Connect actual revm execution state changes to EvmStateDelta without bypassing validation.
2. Connect actual revm execution state changes to EvmStateDelta without bypassing validation.
3. Define a typed explicit execution context supplied by validated block data: fork ID, block number, timestamp, gas limit, base fee, coinbase/fee recipient, chain ID and randomness/prevrandao fields where required. No host-derived values.
5. Ratify the EVM fork and activation schedule after dependency/API and execution-spec review.
6. Implement canonical type-2 transaction decoding and signature recovery before treating sender identity as trusted.
7. Implement EIP-1559 fee settlement: affordability, escrow, actual gas, base-fee burn, priority-fee credit and unused-gas refund with exact overflow/rounding rules.
8. Add canonical receipts/logs and complete state commitments.
9. Run official Ethereum execution-spec fixtures and differential tests against an independent client; fuzz codecs and database conversion.

## Exit evidence

- Official fork-specific execution fixtures and Ethereum test suites.
- Differential outputs against an independent client.
- Reproducible replay with fixed parent state, block context and transactions.
- Full state-root agreement and fee/supply conservation vectors.
- Fuzz/property tests and independent security review.

## Production decision

**NO-GO.** G1 remains blocked. No production chain execution, validator operation or real-funds use is authorized.
