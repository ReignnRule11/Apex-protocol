# Deterministic State Commitments — Phase 14 Prototype

**Status:** account-only SHA-256 Merkle prototype implemented on a review branch; not a complete protocol state root.

## Logical-state boundary

The current state model is limited to AccountId(u64), nonce(u64) and balance(u128). Contract storage, code, system accounts, validator state, receipts, and execution metadata are not included. The prototype must not be represented as the complete APEX Layer-1 state root.

## Canonical account record v1

The 38-byte record is:
- bytes 0–3: ASCII APEX
- bytes 4–5: schema version 1, u16 big-endian
- bytes 6–13: AccountId, u64 big-endian
- bytes 14–21: nonce, u64 big-endian
- bytes 22–37: balance, u128 big-endian

The decoder requires exactly 38 bytes and rejects wrong magic and unknown versions. The redb account record is a separate private storage schema and is not reused as protocol bytes.

## Hash and tree

SHA-256 is pinned through sha2 0.10.9. SHA-256 is part of NIST's Secure Hash Standard (FIPS 180-4). The tree shape follows the recursive construction in RFC 6962 Section 2.1, while all APEX hashing inputs use APEX-specific domain strings.

- Leaf: SHA-256(APEX/STATE/ACCOUNT/LEAF/V1 || canonical account bytes)
- Internal node: SHA-256(APEX/STATE/MERKLE/NODE/V1 || left || right)
- Empty tree: SHA-256(APEX/STATE/MERKLE/EMPTY/V1)
- Root: SHA-256(APEX/STATE/ROOT/V1 || u16_be(1) || u64_be(account_count) || tree_root)

Leaves are ordered by numeric AccountId. The root depends only on the logical account map and schema; it does not include StateVersion, block height, insertion order or database metadata.

## Fixed vector

For accounts (1,0,100), (5,7,500), (9,2,900), where each tuple is (AccountId, nonce, balance), the root is:

fa4b00c9b8ee50bdb2b17109e699df69b29e23cf78737e53674797910f3192d5

## Determinism requirements

Independent implementations must reproduce exact encoded account bytes and roots. The fixed vector is a minimum compatibility gate. Further vectors must cover empty state, one leaf, two leaves, powers of two, non-powers of two, maximum integers, and malformed encodings.

## Open work

No membership/non-membership proof API, proof verifier, incremental tree, persisted commitment index, state sync, EVM storage/code, cross-language implementation, fuzzing campaign or consensus root activation is implemented. RFC 6962 is used as a tree-shape reference, not as a claim of Certificate Transparency compatibility.
