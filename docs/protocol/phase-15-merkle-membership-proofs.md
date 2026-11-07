# Phase 15 — Merkle Membership Proofs

**Status:** merged in PR #14; CI passed formatting, Clippy and workspace tests. Research prototype; not production-approved.

## Scope

Phase 15 adds account membership proof generation and independent verification over the exact Phase 14 account-only root. The Phase 14 root bytes and hash domains are unchanged. This proves that one canonical AccountId/nonce/balance record is included in a committed account map; it does not prove correctness of execution or completeness of the Layer-1 state.

## Proof format

An `AccountMembershipProof` contains:
- the canonical 38-byte account record as structured fields;
- zero-based leaf index and total leaf count;
- a bottom-up sequence of sibling hashes, each explicitly marked Left or Right.

Leaves are in numeric AccountId order. The verifier derives the required path length and sibling directions from (index, count) using the same RFC 6962-style largest-power-of-two split. It rejects zero count, out-of-range indices, non-canonical path shape, and a root mismatch. The top-level wrapper binds schema version and leaf count exactly as Phase 14.

## Complexity

For n accounts, proof generation currently builds all leaf hashes and computes subtree roots recursively; it is a reference implementation, not an optimized index. Proof size and verification work are O(log n) sibling hashes. Generation is O(n) or worse in this straightforward implementation due to repeated subtree hashing. A future persisted/incremental Merkle index is a separate optimization and must preserve byte-for-byte roots.

## Security and limitations

- Proofs authenticate only membership relative to a supplied trusted root. They do not establish that the root itself is valid or finalized.
- The caller must obtain the expected root from a trusted consensus/finality context.
- AccountId remains a u64 prototype identifier; contract storage, code, validators, receipts and other state are excluded.
- Non-membership is not implemented. An absent-account lookup returning no proof is not a cryptographic proof of absence.
- No sparse tree, neighbor proof, multiproof, state sync, proof cache, fuzz campaign, cross-language verifier or consensus root activation is included.
- RFC 6962 is used only for recursive tree shape. APEX uses its own domain strings and is not Certificate Transparency compatible.

## Validation

Tests preserve the Phase 14 fixed root vector; verify every leaf across tree sizes 1 through 17; and reject changed account data, changed siblings, wrong indices, wrong root, wrong directions, truncated paths and empty-tree proofs.
