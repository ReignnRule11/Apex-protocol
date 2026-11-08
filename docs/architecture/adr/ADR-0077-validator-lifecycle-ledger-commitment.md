# ADR-0077 — Deterministic Commitment for the Validator Lifecycle Reference Ledger

- **Status:** Proposed
- **Date:** 2026-11-08
- **Related phase:** 20G2D21
- **Supersedes:** None

## Context

Phase 20G2D20 established deterministic replay auditing for the in-memory validator lifecycle ledger. A stable digest of an audited history is useful for regression checks and future integration design, but only if every field is encoded explicitly and the digest cannot be mistaken for authenticated provenance.

## Decision

Add LifecycleLedger::commitment, which:
1. runs the existing full ledger audit and returns its typed error on failure;
2. encodes scopes in ordered-map order and records in append order;
3. encodes all event variants and lifecycle state fields with explicit discriminants;
4. uses big-endian fixed-width integers and length-prefixes proof bytes;
5. hashes the versioned encoding with SHA-256 and a dedicated domain tag; and
6. returns a 32-byte digest without mutating the ledger.

The event/state discriminants and domain tag are part of this reference encoding version. Any incompatible encoding change requires a new version/domain tag.

## Consequences

### Positive

- Identical audited ledgers produce stable commitments.
- Valid history changes alter the digest.
- A corrupt ledger cannot be committed through this API because audit runs first.
- Explicit encoding avoids relying on Rust Debug, compiler-dependent enum layout, or implicit serialization.

### Constraints

- The digest is not a signature, MAC, authenticated checkpoint, canonical wire format, trusted timestamp, or anti-rollback anchor.
- An actor able to rewrite the history can recompute the digest.
- The supplied protocol proof verifier must be deterministic and trustworthy.
- The implementation remains an in-memory reference facility; no durable storage or external anchoring is introduced.

## Rejected shortcuts

- Hashing Debug output or compiler-dependent enum representations.
- Hashing only current-state indexes without replay.
- Treating an unkeyed SHA-256 digest as authentication.
- Introducing a production persistence or governance decision in this phase.
