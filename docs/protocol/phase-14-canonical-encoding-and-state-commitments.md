# Phase 14 — Canonical Account Encoding and State Commitment Prototype

**Status:** implementation submitted for review; CI pending. Research prototype only.

## Scope

The existing APEX logical state currently consists only of AccountId(u64), nonce(u64) and balance(u128). This phase does not claim a complete Layer-1 state root; EVM contract storage, code, system accounts, validator state, receipts and other protocol state are not represented.

## Canonical account record v1

Exactly 38 bytes:

| Offset | Width | Field | Encoding |
|---:|---:|---|---|
| 0 | 4 | Magic | ASCII APEX |
| 4 | 2 | Schema version | u16 big-endian, value 1 |
| 6 | 8 | AccountId | u64 big-endian |
| 14 | 8 | Nonce | u64 big-endian |
| 22 | 16 | Balance | u128 big-endian |

The decoder rejects any other length, wrong magic or unknown schema version. No trailing bytes or alternate representations are accepted.

## Commitment algorithm v1

- Hash: SHA-256, using sha2 0.10.9 pinned in workspace dependencies.
- Account leaves sorted by numeric AccountId.
- Leaf: SHA-256(LEAF_DOMAIN || canonical_account_record_v1).
- Internal node: SHA-256(NODE_DOMAIN || left || right).
- Empty tree: SHA-256(EMPTY_DOMAIN).
- Tree shape: RFC 6962 recursive split at the largest power of two strictly smaller than the leaf count.
- Top-level root: SHA-256(ROOT_DOMAIN || schema_version_u16_be || account_count_u64_be || tree_root).
- APEX-specific domain strings are distinct for leaves, nodes, empty tree and top-level root.

The root is content-addressed: it excludes StateVersion and block height. The account count is included at the top level. There is no state mutation or storage coupling in the commitment crate.

## Implementation

- apex-codec: AccountRecordV1 and strict encode/decode functions.
- apex-commitments: deterministic account leaf hashing and recursive Merkle root.
- Workspace: new crate member and exact sha2 dependency pin.
- Tests: boundary integer round-trip, byte order, malformed/trailing/version rejection, insertion-order invariance, account-content sensitivity, empty/single/multiple leaves, non-power-of-two tree shape and a fixed three-account cross-implementation vector.

## Fixed test vector

Accounts (sorted by AccountId):
- AccountId 1, nonce 0, balance 100
- AccountId 5, nonce 7, balance 500
- AccountId 9, nonce 2, balance 900

Expected root (hex):
fa4b00c9b8ee50bdb2b17109e699df69b29e23cf78737e53674797910f3192d5

## Explicit non-claims and next gates

No transaction/block codec, account-address model, contract storage/code commitment, inclusion/non-inclusion proof, incremental update tree, state sync, consensus integration, production state root, activation height or audit is delivered. Cross-language reproduction, fuzzing, proof verification and expanded logical state design remain required.
