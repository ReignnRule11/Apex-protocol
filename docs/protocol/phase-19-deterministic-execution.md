# Phase 19 — Deterministic EVM Execution and Fee Accounting

**Status:** Engine evaluation baseline; no EVM engine is pinned or integrated.
**Release impact:** G1 remains BLOCKED.

## 1. Objective

Implement a deterministic EVM execution subsystem that can validate and execute the proposed initial EIP-1559 type-2 transaction profile against a complete versioned parent-state view. Execution must produce canonical receipts, gas accounting, fee settlement and a complete post-state commitment. A library call that executes bytecode is not, by itself, a blockchain execution layer.

## 2. Engine evaluation

### Preferred candidate: revm

The Rust EVM implementation revm is the preferred candidate for a focused integration spike because:
- it is a Rust-native EVM implementation with executor and framework APIs;
- it is used by established Ethereum tooling and clients including Reth and Foundry;
- it exposes Alloy primitives used across the Rust Ethereum ecosystem;
- the upstream project is MIT-licensed;
- its documentation describes execution against explicit block context and transactions.

The latest release surfaced during this evaluation is revm 43.0.3 (2026-09-24). This is a candidate version only, not a dependency selection or production approval.

### Blocking compatibility questions

The upstream project states that it tracks the latest stable Rust and may update its MSRV at any time. APEX currently pins Rust 1.90.0. The exact MSRV and feature graph of the candidate release have not yet been proven against this workspace. A dedicated CI compile spike must establish compatibility before pinning.

The current revm release line includes work targeting Glamsterdam development networks. Ethereum's published roadmap identifies Fusaka/Osaka as deployed and Glamsterdam as in development; the Foundation postponed the Hoodi Glamsterdam activation on 2026-10-22 and had not published a replacement activation date at the time of this evaluation. APEX must not accidentally enable devnet-only or not-yet-finalized fork behavior by selecting a library version.

### Alternatives and decision

Do not select an older EVM implementation solely to preserve the current Rust toolchain. Do not fork revm or implement an EVM from scratch without compelling, independently reviewed evidence. Keep the engine behind an APEX-owned adapter so that upstream types do not become APEX wire formats.

**Decision:** revm is the preferred candidate for a compile/conformance spike. The engine, exact crate version, feature set and EVM fork remain unselected until the gates below pass.

## 3. Fork policy candidate

Use a named, frozen execution ruleset, not a floating "latest Ethereum" mode. The initial candidate baseline is Osaka/Fusaka-era EVM behavior. Glamsterdam and later upgrades are excluded until the upstream specifications are finalized, the chosen revm release supports the final rules, and APEX ratifies explicit activation semantics.

This fork candidate is separate from transaction-type support: the initial APEX transaction profile remains type 0x02 only unless a later ADR expands it.

## 4. Required adapter boundary

The future apex-evm crate must own:
- conversion between APEX versioned state and engine database interfaces;
- chain ID and block execution context;
- transaction decoding/authentication handoff;
- fork selection by deterministic protocol parameters;
- transaction execution and revert semantics;
- gas and fee settlement inputs/outputs;
- canonical receipt and log conversion;
- state-diff extraction and atomic application handoff.

The adapter must not allow engine-specific serialized structs to become canonical APEX protocol formats. It must not read host time, random values, local files or network state during deterministic execution.

## 5. Execution invariants

For identical parent state, block context, transaction bytes and fork parameters, all conforming nodes must produce identical:
- transaction acceptance/rejection;
- sender and destination semantics;
- execution status and return/revert data;
- gas used and cumulative block gas;
- ordered logs and receipts;
- effective gas price and fee allocations;
- state changes and complete post-state root.

A failed transaction must follow the selected EVM fork's exact rollback semantics. Transaction-level reverts do not imply block invalidity; invalid block structure or an invalid transaction envelope rejects the block. Fee debits, nonce consumption and refunds must follow the pinned execution rules, not APEX-specific approximations.

## 6. Fee-accounting boundary

The type-2 transaction has max fee and priority fee caps. Actual charged fee must be derived from the active base fee and effective priority fee under the selected EIP-1559 rules. The protocol must account exactly for:
- sender upfront affordability and escrow;
- actual gas used;
- base-fee burn;
- proposer priority-fee allocation;
- refund of unused gas escrow;
- overflow and integer-rounding behavior.

The current transfer planner's maximum-fee escrow is only an admission prototype. It is not production fee settlement and must not be reused as if it were.

## 7. Required conformance evidence

- Run the official Ethereum execution-spec state, blockchain and transaction fixtures relevant to the pinned fork.
- Run the upstream Ethereum tests repository's applicable GeneralStateTests, BlockchainTests, TransactionTests, EIPTests and RLP tests.
- Differentially compare results against at least one independent execution client for the selected fork.
- Include EVM state tests for storage, creation, calls, reverts, logs, gas boundaries, precompiles, warm/cold access, self-destruct semantics and fork transitions.
- Verify EIP-1559 fee calculations and supply accounting with independent vectors.
- Replay identical block fixtures on different supported architectures and compare receipts and complete state roots.
- Fuzz transaction decoding, engine database adapters and state-diff conversion.

Passing revm's own tests is necessary but not sufficient; APEX integration and cross-client differential evidence are required.

## 8. Phase 19 exit gates

1. Prove the exact candidate revm release builds on the pinned APEX toolchain or explicitly approve a toolchain update.
2. Complete dependency, license, feature and security review.
3. Ratify a named EVM fork and activation model.
4. Implement a minimal apex-evm adapter without exposing engine-specific wire types.
5. Pass official execution fixtures and differential tests.
6. Implement deterministic fee settlement and complete-state transition integration.
7. Demonstrate deterministic replay and bounded resource use.

Until these gates pass, no EVM execution, Ethereum equivalence or fee-market correctness is claimed. G1 remains BLOCKED.
