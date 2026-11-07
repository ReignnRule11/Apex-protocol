# Phase 19 — Deterministic EVM Execution and Fee Accounting

**Status:** revm 43.0.3 pinned; staged state, storage, execution-result, environment and builder adapters merged/in progress.
**Release impact:** G1 remains BLOCKED.

## Objective

Implement deterministic EVM execution against a complete versioned parent-state view. Execution must eventually produce canonical receipts, gas accounting, fee settlement and a complete post-state commitment. A library call that executes bytecode is not, by itself, a blockchain execution layer.

## Engine candidate

- Engine: revm 43.0.3, exact version pin in `crates/apex-evm/Cargo.toml`.
- Toolchain: Rust 1.91.0.
- Features: default features disabled; std enabled explicitly.
- Fork: no APEX fork is ratified.
- Adapter: APEX-owned; revm types are not canonical wire or storage formats.

## Progress

- **19A–19B:** revm compatibility spike and minimal test-only execution harness.
- **19C–19E:** full-width state model, read-only revm Database adapter and atomic in-memory state-delta application.
- **19F–19H:** versioned redb persistence prototype and selected corruption/commit-ambiguity recovery tests. These do not prove real power-loss durability or production storage suitability.
- **19I:** revm state-diff extraction into deterministic parent-version-bound APEX deltas.
- **19J:** success-gated extraction; revert and halt outcomes are rejected.
- **19K:** immutable APEX block execution context and validated fork activation schedule.
- **19L:** explicit context-to-revm environment mapping (merged). Requires a matching APEX ForkId/revm SpecId binding; rejects lossy 256-bit-to-u64 conversions; supports only post-Merge/pre-Cancun profiles; rejects missing prevrandao and unsupported fork profiles. Uses revm's mainnet gas table as a provisional compatibility mapping, not approved APEX gas policy.
- **19M:** explicit revm builder integration (merged). Builds the engine from the APEX immutable snapshot and the exact Phase 19L CfgEnv/BlockEnv. No default context or latest fork is selected; no APEX transaction is authenticated or executed yet.

## Phase 19L limitations

The environment adapter does not authenticate the source block/configuration or execute a transaction. revm 43.0.3 uses u64 chain ID and base fee fields; values outside that range fail closed. Blob-gas inputs, pre-Merge difficulty, Amsterdam slot semantics, fork-specific APEX gas rules, transaction authentication, fee settlement, receipts and full-state commitments remain outstanding.

## Phase 19M limitations

The builder selects revm's standard Ethereum instruction/precompile implementation through the `MainBuilder::build_mainnet` API, but receives the exact explicit APEX `CfgEnv`, `BlockEnv` and snapshot database first. This API name does not mean `Context::mainnet()` is called or a default fork is chosen. Transaction decoding/authentication, canonical TxEnv construction, transaction execution, fee settlement, receipts and atomic persistent state application remain unimplemented.

## Phase 19N — validated type-2 transaction environment (in progress)

Adds an APEX-owned AuthenticatedType2Transaction execution input and bounded conversion to revm 43.0.3 TxEnv. It maps caller, nonce, chain ID, max fee, priority fee, gas limit, call/create destination, value, input and access list. Chain ID must match the explicit execution environment. Zero chain ID/gas limit, invalid fee ordering and caller-supplied input/access-list limit violations fail closed. This does not decode RLP, verify signatures, recover senders, perform state admission, execute transactions or settle fees. The public constructor relies on an upstream authentication precondition; it cannot prove provenance.

## Required next work

1. Ratify APEX fork semantics, activation rules and the explicit mapping to revm SpecIds.
2. Resolve 256-bit APEX chain-ID/base-fee versus revm u64 limits.
3. Extend context and mapping for blob gas and future fork-specific environment fields before enabling those forks.
4. Connect the explicit environment to the EVM builder and authenticated transaction pipeline without mainnet defaults.
5. Implement canonical type-2 transaction decoding and signature recovery.
6. Implement exact fee settlement, canonical receipts/logs and complete state commitments.
7. Run official execution-spec fixtures, differential tests, fuzzing and independent review.

## Exit evidence

- Official fork-specific execution fixtures and Ethereum test suites.
- Differential outputs against an independent client.
- Reproducible replay with fixed parent state, block context and transactions.
- Full state-root agreement and fee/supply conservation vectors.
- Fuzz/property tests and independent security review.

## Production decision

**NO-GO.** G0 and G1 remain blocked. No production chain execution, validator operation or real-funds use is authorized.
