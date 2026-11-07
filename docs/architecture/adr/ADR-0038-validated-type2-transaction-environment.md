# ADR-0038: Validated Type-2 Transaction to revm TxEnv

**Status:** Proposed pending PR validation and review
**Date:** 2026-11-07

## Context

Phase 19M constructs revm from an explicit APEX block environment and immutable state snapshot. The next boundary is transaction environment construction. APEX has a proposed type-2-only envelope profile, but no canonical decoder or signature verifier.

## Decision

Add an APEX-owned execution-facing AuthenticatedType2Transaction representation and convert it to revm 43.0.3 TxEnv. The input carries a recovered caller, chain ID, nonce, EIP-1559 fee fields, gas limit, destination/create marker, value, input and access list.

The adapter requires exact chain-ID agreement with RevmExecutionEnvironment. It rejects zero chain ID/gas limit, priority fee above max fee, and resource-limit excesses. Resource bounds are explicit caller configuration, not protocol constants.

## Trust and scope

The name "authenticated" documents a precondition; the public constructor does not cryptographically prove it. Only an upstream component that has decoded the canonical signed envelope, verified its signature and recovered the sender may supply trusted values. Until that component exists, tests use synthetic values only.

The adapter does not decode RLP, verify signatures, derive transaction IDs, perform state admission, execute, settle fees, produce receipts or commit state. revm TxEnv is not a canonical wire format.

## Consequences

- Transaction fields enter revm through a bounded, chain-bound adapter.
- Unsupported or over-limit inputs fail closed.
- Chain-ID and fee-field width limits remain explicit.
- G0/G1 remain blocked.

## Alternatives considered

1. Populate TxEnv directly at call sites — rejected because it spreads mapping and validation rules.
2. Trust a supplied sender without documenting provenance — rejected.
3. Claim the adapter verifies authentication — rejected; no cryptographic verifier exists.
