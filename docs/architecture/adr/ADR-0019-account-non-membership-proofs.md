# ADR-0019: Ordered-Merkle Account Non-Membership Proofs

- **Status:** Accepted for research prototype; production protocol selection withheld
- **Date:** 2026-11-07

## Context

Phase 15 proves inclusion in the Phase 14 account-only Merkle root. Consumers also need to prove that a key is absent without changing the committed root format.

## Decision

1. Preserve the existing ordered binary Merkle tree and Phase 14 root bytes.
2. Prove absence using authenticated predecessor/successor membership proofs.
3. Support empty-tree, before-first, after-last, and between-neighbor cases.
4. Verify every neighbor against the same expected root; require matching leaf counts, exact boundary indices or adjacent indices, and strict AccountId ordering around the query.
5. Refuse to generate a non-membership proof when the queried account exists.
6. Keep production protocol selection withheld pending full-state design, cross-implementation vectors, fuzzing, and independent review.

## Consequences

Absence can be verified with at most two existing membership proofs and O(log n) hash verification per neighbor. Root format remains unchanged. The proof establishes absence only in the account-only ordered set represented by a trusted root; it does not authenticate root finality or cover state omitted from the prototype.
