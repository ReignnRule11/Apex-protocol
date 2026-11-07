# Phase 19B — Minimal deterministic EVM execution harness

**Status:** Implementation submitted for CI validation.
**Release impact:** G1 remains BLOCKED.

## Objective

Move beyond dependency compilation and exercise the pinned revm candidate through an actual EVM execution path. The first fixture deploys a tiny contract into a fresh in-memory database, writes a known value to storage during creation, then calls the deployed contract and inspects the resulting state diff.

## Implementation

- Uses the exact `revm = 43.0.3` dependency already pinned in `crates/apex-evm`.
- Uses revm's documented execution API: `Context`, `CacheDB<EmptyDB>`, `MainBuilder`, `TxEnv`, `ExecuteCommitEvm` and `ExecuteEvm`.
- Executes contract creation and commits the creation result to the fixture's in-memory database.
- Executes a call against the created contract and verifies slot zero in the returned state diff equals `0x42`.
- Runs the fixture twice from fresh contexts and asserts identical observed execution output.
- Keeps all fixture code in unit tests; no production RPC or node surface is exposed.

## Important scope limits

This is a minimal API/behavior harness, not an APEX block executor. It currently uses revm's mainnet context defaults and does not supply a ratified APEX fork schedule or explicit production block environment. The fixture does not prove cross-architecture determinism, EVM fork conformance, authenticated transaction handling, complete database semantics, or production gas/fee settlement.

It does not implement:
- APEX 20-byte address/state database adapter or complete account/storage/code model;
- canonical RLP decoding, signature recovery, chain-ID checks or replay protection;
- consensus-derived block number, timestamp, coinbase, gas limit or base fee;
- APEX transaction validation and EIP-1559 fee escrow/burn/tip/refund;
- canonical receipts/logs or complete post-state commitments;
- official Ethereum execution fixtures or cross-client differential testing.

## Why the fixture is deliberately narrow

The previous APEX state crates use an internal `AccountId(u64)` and `u128` balance prototype. The protocol draft requires 20-byte EVM addresses and 256-bit balances plus contract storage and code. Adapting the current prototype directly would silently narrow the EVM state model. This phase therefore proves the pinned engine's execution API before introducing a separate full-width EVM state model.

## Validation required

CI must pass formatting, Clippy and all workspace tests. The next phase must replace the fixture-only database with an APEX-owned full-width state adapter and explicit, versioned execution context. Before production consideration, run the official execution-spec fixtures for a ratified fork and differential tests against an independent client.

## Upstream API reference

The fixture follows the upstream revm v43.0.3 contract-deployment example (release tag v120): https://github.com/bluealloy/revm/blob/v120/examples/contract_deployment/src/main.rs
