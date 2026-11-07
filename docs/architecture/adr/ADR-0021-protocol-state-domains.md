# ADR-0021: Complete Protocol State Domains and Versioned Envelope Boundaries

- **Status:** Accepted as architecture constraints; wire formats and cryptographic algorithms remain open
- **Date:** 2026-11-07

## Context

Phases 4–16 built bounded prototypes around AccountId(u64), nonce(u64), balance(u128), a 38-byte account record and an account-only SHA-256 Merkle root. These prototypes do not represent EVM state, validator/staking state, receipts, governance or monetary accounting. Promoting them directly would create an incompatible and incomplete production state model.

## Decisions

1. The production target is EVM-compatible and requires 20-byte account addresses and 256-bit EVM values. AccountId(u64) remains a prototype-only identifier.
2. The production state model is multi-domain: EVM accounts, contract code/storage, validator registry, staking, monetary accounting, consensus parameters, governance, receipts and protocol metadata.
3. The production state commitment MUST cover every state domain that can affect future validity, execution, consensus membership, monetary supply or governance. Reconstructible derived indexes may be excluded only when their omission cannot affect validity.
4. Historical Phase 14–16 account roots and proofs remain versioned research artifacts. They MUST NOT be silently reinterpreted as the production state root.
5. Transaction, block, receipt, genesis and consensus evidence formats MUST be independently versioned, canonical, bounded and chain-bound. No general-purpose encoding or signature algorithm is selected by this ADR.
6. Cryptographic roles MUST have separate domains and separately managed keys. EVM transaction, consensus signing and peer identity keys MUST NOT be reused by assumption.
7. All consensus-critical decisions left open in Protocol Specification v0.2 require explicit ADRs, conformance vectors and review before G0 can pass.

## Consequences

The production account codec and complete state commitment require a new design. The existing transaction planner and storage adapter remain useful prototypes but cannot be wired into a production consensus engine without replacing or extending their data model and adding authentication, execution, receipts and cross-subsystem atomicity. This decision intentionally prevents prototype compatibility from constraining the production protocol incorrectly.
