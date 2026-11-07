# Phase 19 — Deterministic EVM Execution and Fee Accounting

**Status:** revm 43.0.3 candidate pinned; execution harness, full-width state model, revm read adapter and atomic in-memory delta application staged.
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
Introduces ordered before/after account, code and storage updates bound to a parent version. A process-local in-memory store validates all preconditions and the complete resulting snapshot before publishing a replacement. It is not persistent, concurrent or crash-safe. See phase-19e-atomic-evm-state-delta.md and ADR-0029.

## Required next work

1. Design a persistent full-width state schema and atomic commit/recovery semantics.
2. Connect actual revm execution state changes to the APEX EvmStateDelta representation.
3. Define a typed explicit execution context supplied by validated block data: fork ID, block number, timestamp, gas limit, base fee, coinbase/fee recipient, chain ID and randomness/prevrandao fields where required. No host-derived values.
4. Ratify the EVM fork and activation schedule after dependency/API and execution-spec review.
5. Implement canonical type-2 transaction decoding and signature recovery before treating sender identity as trusted.
6. Implement EIP-1559 fee settlement: affordability, escrow, actual gas, base-fee burn, priority-fee credit and unused-gas refund with exact overflow/rounding rules.
7. Add canonical receipts/logs and complete state commitments.
8. Run official Ethereum execution-spec fixtures and differential tests against an independent client; fuzz codecs and database conversion.

## Exit evidence

- Official fork-specific execution fixtures and Ethereum test suites.
- Differential outputs against an independent client.
- Reproducible replay with fixed parent state, block context and transactions.
- Full state-root agreement and fee/supply conservation vectors.
- Fuzz/property tests and independent security review.

## Production decision

**NO-GO.** G1 remains blocked. No production chain execution, validator operation or real-funds use is authorized.
