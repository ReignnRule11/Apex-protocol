# ADR-0026: First executable revm integration harness

- **Status:** Proposed pending CI validation
- **Date:** 2026-11-07
- **Related:** ADR-0024, ADR-0025, Phase 19B

## Context

The Phase 19A spike proves the exact revm 43.0.3 dependency graph builds under Rust 1.91.0. A dependency compile alone does not prove APEX can call the execution API.

## Decision

Add a test-only EVM fixture inside `apex-evm`. It deploys a minimal contract into a fresh revm in-memory database, writes a known value to storage, calls the contract and checks the execution state diff. A second fresh run checks repeatability of the observed output.

The fixture follows the upstream v43.0.3 contract-deployment example and remains test-only. It does not expose revm types as APEX protocol types.

## Consequences

- CI now exercises contract creation, state commit to a local test database, a contract call and storage-state extraction.
- The candidate engine is exercised through its actual execution API rather than only linked.
- The test still relies on revm mainnet context defaults and is not consensus-ready.
- G1 remains BLOCKED until APEX has a full-width state adapter, explicit fork/block context, authenticated transactions, exact fee accounting, canonical receipts and complete state commitments, plus official and differential fixtures.

## Rejected shortcuts

- Treating a test-only in-memory database as the production state backend.
- Reusing the historical `AccountId(u64)`/u128 prototype as the EVM account model.
- Treating two identical local runs as cross-client or cross-architecture determinism evidence.
- Enabling node execution before protocol fork and context parameters are ratified.
