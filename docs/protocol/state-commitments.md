# Deterministic State Commitments — Account-Only Prototype

**Status:** Phase 14 root plus Phase 15 membership and Phase 16 non-membership proof prototypes; not a complete protocol state root.

## Logical-state boundary

The current state model is limited to AccountId(u64), nonce(u64) and balance(u128). Contract storage, code, system accounts, validator state, receipts, and execution metadata are not included. This prototype must not be represented as the complete APEX Layer-1 state root.

## Canonical account record v1

Exactly 38 bytes:
- bytes 0–3: ASCII APEX
- bytes 4–5: schema version 1, u16 big-endian
- bytes 6–13: AccountId, u64 big-endian
- bytes 14–21: nonce, u64 big-endian
- bytes 22–37: balance, u128 big-endian

The decoder requires exactly 38 bytes and rejects wrong magic and unknown versions. The redb account record is a separate private storage schema.

## Hash and tree

SHA-256 is pinned through sha2 0.10.9. Leaves are ordered by numeric AccountId. APEX-specific domains are used:
- Leaf: SHA-256(APEX/STATE/ACCOUNT/LEAF/V1 || canonical account bytes)
- Internal node: SHA-256(APEX/STATE/MERKLE/NODE/V1 || left || right)
- Empty tree: SHA-256(APEX/STATE/MERKLE/EMPTY/V1)
- Root: SHA-256(APEX/STATE/ROOT/V1 || u16_be(1) || u64_be(account_count) || tree_root)

The tree shape follows RFC 6962 Section 2.1's recursive split, but APEX is not Certificate Transparency-compatible. The root does not include StateVersion, block height, insertion order or database metadata.

## Membership proofs (Phase 15)

`account_membership_proof_v1` returns an inclusion proof for an existing AccountId. The proof contains the canonical account fields, zero-based leaf index, leaf count and bottom-up sibling hashes with Left/Right direction.

`verify_account_membership_proof_v1` reconstructs the expected path shape from index and count, rejects malformed paths and compares the reconstructed wrapped root with the caller-supplied expected root. The verifier does not trust proof-provided direction metadata.

Verification requires O(log n) hashes and proof size O(log n). Current proof generation rebuilds leaf hashes and subtree roots as a simple reference implementation; it is not optimized for large state maps.

The verifier proves inclusion only relative to a supplied root. It does not establish that the root is finalized or valid under consensus.

## Non-membership proofs (Phase 16)

`account_non_membership_proof_v1` proves absence in the numerically ordered AccountId set by composing authenticated neighbor membership proofs. Empty state uses the canonical empty root; boundary cases prove the first or last leaf; interior absence proves adjacent predecessor and successor indices. Both neighbors must verify against the same expected root, have matching leaf counts, and strictly bracket the queried AccountId. Generation refuses existing accounts.

This proves absence only in the account-only set represented by a trusted root. It does not authenticate root finality or cover omitted protocol state. The implementation remains a reference prototype without an optimized index or independent audit.

## Fixed vector

For accounts (1,0,100), (5,7,500), (9,2,900), where each tuple is (AccountId, nonce, balance), the Phase 14 root remains:

`fa4b00c9b8ee50bdb2b17109e699df69b29e23cf78737e53674797910f3192d5`

## Deferred work

No multiproof, sparse Merkle tree, incremental tree, persisted commitment index, state sync, complete protocol state root, EVM storage/code, cross-language verifier, fuzzing campaign or consensus root activation is implemented. An absent lookup returning no proof is not proof of absence.
