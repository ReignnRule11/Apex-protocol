# Phase 19N — Validated Type-2 Transaction Environment

**Status:** Implementation on branch `phase-19n/validated-transaction-environment`; validation pending.
**Release impact:** G0 and G1 remain BLOCKED.

## Objective

Map fields from an already authenticated, validated type-2 transaction into revm 43.0.3 `TxEnv`, bound to the explicit APEX execution chain ID.

## Implementation

- Add `apex-evm::revm_transaction` and an execution-facing `AuthenticatedType2Transaction`.
- Map transaction type 0x02, recovered caller, nonce, chain ID, EIP-1559 max/priority fees, gas limit, call/create destination, value, input and access list.
- Require nonzero chain ID and gas limit, priority fee no greater than max fee, and exact chain-ID equality with the Phase 19L execution environment.
- Enforce caller-supplied maximum input bytes, access-list addresses and access-list storage keys. These are explicit adapter bounds, not ratified consensus constants.
- Preserve access-list order and values. revm TxEnv remains an internal adapter, not a canonical wire format.

## Trust boundary and limitations

The type name documents a precondition; the public constructor does not cryptographically prove authentication. Only an upstream component that has decoded the canonical signed envelope, verified its signature and recovered the sender may supply trusted values.

This phase does not decode RLP, verify secp256k1 signatures, compute signing digests or transaction IDs, perform state admission, execute transactions, settle fees, produce receipts, or commit state. The current revm boundary retains u64 chain-ID and u128 fee widths. No silent truncation is permitted.

## Tests

Call mapping, contract creation, wrong-chain rejection, invalid fee ordering, zero gas limit, and configured input/access-list bounds. Exact-head GitHub CI is authoritative for compile, Clippy and test evidence.

## Exit criteria

G1 still requires canonical decoding, signature recovery, official vectors, state admission, deterministic execution, exact fee settlement, canonical receipts and full-state commitments, differential tests, fuzzing and independent review.

**Decision: NO-GO.** No production transaction processing or real-funds use is authorized.
