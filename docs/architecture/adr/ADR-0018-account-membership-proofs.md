# ADR-0018: Account Membership Proofs

- **Status:** Accepted for research prototype; production protocol selection withheld
- **Date:** 2026-11-07

## Context

Phase 14 defines an account-only content-addressed root over canonical account records. Consumers need a verifiable way to check whether one account record is included without receiving the full map.

## Decision

1. Add membership proof generation and verification in `apex-commitments`.
2. Preserve the Phase 14 root construction, schema version, hash domains, SHA-256, leaf ordering and RFC 6962-style recursive split.
3. Encode proof metadata as leaf index, leaf count, and bottom-up sibling hashes with explicit side markers.
4. The verifier independently derives expected path directions and length from index/count; proof-supplied directions are checked, not trusted.
5. Keep non-membership, multiproofs, sparse Merkle trees and consensus activation deferred.

## Consequences

Membership can be verified in O(log n) sibling hashes against a caller-supplied trusted root. Proof generation is currently an unoptimized reference path that hashes the full account set. A proof does not authenticate the root's finality, establish execution correctness, or cover state omitted from the current account model.
