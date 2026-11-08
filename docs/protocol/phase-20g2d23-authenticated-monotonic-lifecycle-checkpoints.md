# Phase 20G2D23 — Authenticated Monotonic Lifecycle Checkpoints

## Objective

Define a fail-closed acceptance boundary that binds an audited lifecycle-ledger commitment to an authenticated, network-scoped, sequential checkpoint chain.

## Implementation

- `AuthenticatedLifecycleCheckpoint` carries network ID, sequence, ledger commitment, previous checkpoint hash and signature.
- `signing_bytes()` defines a domain-separated, fixed-width signing payload with big-endian sequence encoding.
- `checkpoint_hash()` commits to the signing payload and length-prefixed signature.
- `LifecycleCheckpointAuthenticator` delegates signature verification to a configured external implementation.
- `MonotonicLifecycleCheckpointStore` defines durable load and atomic compare-and-advance operations.
- `accept_lifecycle_checkpoint` authenticates the checkpoint, recomputes the audited ledger commitment, enforces genesis/next-sequence and previous-head linkage, then performs the atomic store advance.
- Typed errors distinguish invalid network/sequence/signature, ledger failure, commitment mismatch, authentication failure, store failure, non-monotonic sequence, previous-head mismatch and concurrent advancement.

## Invariants

- Genesis is sequence 1 with an all-zero previous checkpoint hash.
- Each subsequent checkpoint is exactly the prior trusted sequence plus one.
- Each subsequent checkpoint links to the exact hash of the prior accepted checkpoint.
- The signed payload binds the network ID, sequence, commitment and previous checkpoint hash.
- The ledger commitment is recomputed through the audited Phase 20G2D21 path before acceptance.
- Store advancement is compare-and-advance; a stale concurrent writer fails closed.
- Rejected checkpoints do not advance the store.

## Tests

- Accept and persist authenticated genesis checkpoint.
- Reject commitment mismatch.
- Reject invalid authentication without store mutation.
- Reject skipped genesis sequence.
- Reject previous checkpoint mismatch.
- Reject stale sequence without mutating the trusted head.

## Critical production boundary

This phase does **not** implement a production signature scheme, key registry, signer quorum, key rotation, revocation, hardware custody, durable database, trusted clock, or external anti-rollback anchor. The authenticator must enforce a ratified signer/key policy and bind it to the intended network/domain. The store must provide atomic durable compare-and-advance and resist rollback even after backup restoration or storage replacement; ordinary database persistence/CAS alone cannot prove that property.

The included test store is in-memory and test-only. It is not a production store. No production authentication or persistence implementation is selected or claimed.

## Release posture

This remains a protocol interface/reference implementation. It does not activate validators or mutate the live consensus validator set. G0–G5 remain blocked; G6/G7 remain not started; overall release remains **NO-GO**.
