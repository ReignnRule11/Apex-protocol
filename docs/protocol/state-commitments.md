# Deterministic State Commitments — Design Gate

**Status:** requirements and candidate structures only; no state root is implemented.

## Why a separate commitment layer is required

A storage engine protects and retrieves local data. A protocol state commitment allows independent validators to derive and compare a cryptographic commitment to the same logical state. Database page hashes, checksums, transaction IDs, file roots, and physical key ordering are not consensus commitments.

The current codec intentionally fails closed, canonical encoding is unresolved in ADR-0005, and `AccountId(u64)` is a prototype-only key. Therefore APEX cannot yet define reproducible state-root bytes.

## Required specification decisions

Before implementation, a dedicated ADR must specify:

1. **Logical state:** exact account fields, contract storage model, code representation, system accounts, and inclusion/exclusion rules.
2. **Canonical keys and values:** byte-level account/storage key formats, integer widths, endianness, optional-field semantics, and canonical encoding version.
3. **Ordering:** total byte ordering of all keys and deterministic duplicate handling.
4. **Hash suite:** exact collision-resistant hash algorithm and version, output size, test vectors, and migration/activation rules.
5. **Domain separation:** distinct prefixes for account leaves, storage leaves, internal nodes, empty values, and top-level state commitment.
6. **Authenticated structure:** evaluate sparse Merkle tree, Merkle Patricia structure, and other authenticated key-value structures against proof size, update complexity, state growth, implementation complexity, and cross-language support.
7. **Empty and absent semantics:** distinguish absent account, zero-valued account, empty contract storage, and empty global state.
8. **Proof format:** membership and non-membership proof structure, verification rules, bounded decoding, and version binding.
9. **Upgrade semantics:** activation height/version, old/new root coexistence if required, and explicit rejection of unknown versions.

## Candidate structure comparison

| Structure | Potential strengths | Costs and risks |
|---|---|---|
| Sparse Merkle tree | Fixed-depth key path; natural membership/non-membership proofs; update paths are explicit | Potentially large proofs; hashing/update cost; key hashing and empty-node rules must be exact |
| Merkle Patricia tree | Authenticated key-value map; compact paths for sparse key spaces; familiar in EVM ecosystems | Complex canonical node encoding and proof rules; careful normalization required |
| Sorted-leaf Merkle tree | Conceptually simple deterministic construction | Updating can be expensive; insertions can shift leaf positions; proof and index semantics need definition |

No structure is selected. An implementation benchmark alone cannot settle protocol correctness.

## Determinism invariants

For a fixed commitment version and identical logical state, every conforming implementation must produce exactly the same root bytes. Insertion order, database engine, host architecture, map implementation, thread schedule, and physical storage layout must not affect the root.

A canonical test-vector suite must include empty state, one account, multiple accounts in non-sorted insertion order, zero values, maximum integer values, absent-key proofs, storage slots, and malformed/non-canonical inputs.

## Required separation of concerns

- Canonical codec defines protocol bytes.
- State model defines logical records.
- Commitment implementation maps logical state to authenticated root/proofs.
- Storage adapter persists logical state and snapshots.
- Execution computes logical changes.
- Consensus accepts only a root calculated under the activated commitment version.

A backend migration must preserve roots for the same logical state. A commitment algorithm migration is a protocol upgrade, not a database migration.

## Explicit non-claims

No APEX state root, Merkle proof, state sync, authenticated storage, or cross-validator commitment is implemented by this document.
