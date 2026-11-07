# Phase 19 — Deterministic EVM Execution and Fee Accounting

**Status:** Candidate engine pinned for isolated integration; minimal test execution harness submitted.
**Release impact:** G1 remains BLOCKED.

## Objective

Implement a deterministic EVM execution subsystem for the proposed initial EIP-1559 type-2 transaction profile against a complete versioned parent-state view. Execution must produce canonical receipts, gas accounting, fee settlement and a complete post-state commitment. A library call that executes bytecode is not, by itself, a blockchain execution layer.

## Engine candidate

- Engine: revm 43.0.3, exact version pin in `crates/apex-evm/Cargo.toml`.
- Toolchain: Rust 1.91.0, exact workspace and CI pin.
- Features: default features disabled; `std` enabled explicitly.
- Upstream license: MIT; APEX still requires a complete dependency/license inventory and security review.
- Fork: not ratified. Osaka/Fusaka-era behavior remains a candidate baseline; Glamsterdam remains excluded until finalized and explicitly selected.
- Adapter: APEX-owned boundary; engine types are not canonical wire/storage formats.

## Progress

### 19A — dependency and toolchain compatibility

The initial Rust 1.90.0 CI attempt failed because the revm 43.0.3 dependency graph declares a Rust 1.91.0 minimum. The workspace metadata, `rust-toolchain.toml` and CI were aligned to Rust 1.91.0. The updated compatibility CI passed and PR #21 was merged.

### 19B — minimal execution API harness

A test-only fixture now deploys a small contract into a fresh revm in-memory database, commits its creation, calls it, and checks a known storage value. A second fresh run checks repeatability of the observed result. This proves that the pinned candidate can be invoked through its execution API for this narrow fixture.

It does not prove APEX block execution, production context, fork correctness, state adapter correctness, fee settlement or cross-client determinism. See `phase-19b-evm-execution-harness.md` and ADR-0026.

## Required next work

1. Design the full-width EVM state model: 20-byte addresses, U256 balances/nonces as required by the selected fork, code hashes/code storage, 256-bit storage keys/values, account existence semantics and bounded reads/writes.
2. Implement an APEX-owned revm database adapter over an immutable versioned state snapshot. Keep all reads anchored to one parent state version; return explicit errors for missing/corrupt state.
3. Define a typed explicit execution context supplied by validated block data: fork ID, block number, timestamp, gas limit, base fee, coinbase/fee recipient, chain ID and randomness/prevrandao fields where required. No host-derived values.
4. Ratify the EVM fork and activation schedule after dependency/API and execution-spec review.
5. Implement canonical type-2 transaction decoding and signature recovery before treating sender identity as trusted.
6. Implement EIP-1559 fee settlement: affordability, escrow, actual gas, base-fee burn, priority-fee credit and unused-gas refund with exact overflow/rounding rules.
7. Convert revm state diffs into atomic APEX state updates; add canonical receipts/logs and complete state commitments.
8. Run official Ethereum execution-spec fixtures and differential tests against an independent client; fuzz codecs and database conversion.

## Exit evidence

- Official fork-specific execution fixtures and Ethereum test suites.
- Differential outputs against an independent client.
- Reproducible replay with fixed parent state, block context and transactions.
- Full state-root agreement and fee/supply conservation vectors.
- Fuzz/property tests and independent security review.

## Production decision

**NO-GO.** G1 remains blocked. No production chain execution, validator operation or real-funds use is authorized.
