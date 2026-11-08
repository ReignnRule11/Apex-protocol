# ADR-0079 — Authenticated Monotonic Validator Lifecycle Checkpoints

- **Status:** Proposed
- **Date:** 2026-11-08
- **Related phase:** 20G2D23
- **Supersedes:** None

## Context

Phase 20G2D22 verifies a recomputed lifecycle commitment against a caller-supplied digest, but does not authenticate the digest's publisher or establish freshness and anti-rollback. Consumers need a network-scoped checkpoint chain whose payload is authenticated and whose trusted head advances atomically.

## Decision

Introduce a checkpoint payload with a domain-separated signing message, sequence, ledger commitment, previous-checkpoint hash and signature. Require an injected authenticator and a monotonic store contract. Accept only genesis sequence 1 or the exact next sequence linked to the current trusted head. Recompute and audit the lifecycle commitment before atomically advancing the head with compare-and-advance.

## Consequences

- Checkpoint payloads are bound to a network ID and protocol domain.
- Sequence gaps, replayed checkpoints and previous-head divergence fail closed.
- Concurrent writers cannot both advance from the same expected head when the store honors its atomic contract.
- Authentication and durable anti-rollback remain explicit external dependencies.
- The in-memory test store is not suitable for production.

## Rejected shortcuts

- Treating a matching unkeyed digest as proof of publisher identity.
- Accepting arbitrary higher sequence numbers without exact linkage.
- Updating a checkpoint head using a non-atomic read-then-write.
- Claiming database persistence alone provides anti-rollback after backup restoration.
- Selecting a signature scheme, signer quorum or custody system without protocol/security ratification.
