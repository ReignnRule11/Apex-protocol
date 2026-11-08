# Phase 20G2D20 — Executable Validator Lifecycle Ledger Audit

## Status

Implementation proposal. This phase adds deterministic replay auditing to the policy-neutral, in-memory validator lifecycle reference ledger introduced in Phase 20G2D19. It does not authorize validator admission, activate consensus membership, or establish a production registry.

## Objective

Provide an executable integrity check that recomputes lifecycle state from the append-only event history and verifies that the ledger's current-state and request-id indexes exactly match that history.

## Implemented contract

- Replay each scope independently from the default lifecycle state.
- Require each record's scope to match its history partition.
- Require each envelope's expected revision to equal the replayed revision.
- Reject zero or duplicate request IDs in the replayed history.
- Reapply each event through the same transition function used by normal ledger application.
- Re-run the injected protocol-transition verifier for activation and exit confirmation records.
- Require each stored resulting state to equal the state recomputed from its event.
- Require the current-state index to equal the replayed final states.
- Require the request-id index to equal the envelopes found in history.
- Fail closed on any mismatch with typed lifecycle errors.

## Validation

The test suite covers:
- consistent multi-event replay;
- tampered resulting-state detection;
- missing request-index detection;
- current-state-index divergence;
- protocol proof re-verification during audit.

The audit is deterministic only to the extent that the injected `ProtocolTransitionVerifier` is deterministic for the same scope, transition ID, height and proof.

## Explicit limitations

This remains an in-memory, single-process reference ledger. The audit does not:
- authenticate the origin or integrity of the history source;
- provide persistence, crash recovery, cross-process locking or distributed ordering;
- define a canonical serialization or wire format;
- supply a production protocol proof verifier;
- authorize governance decisions or signer operations;
- mutate the live consensus validator set;
- establish anti-rollback protection or a trusted monotonic checkpoint.

A successful audit means only that the in-memory history and its derived indexes are internally consistent under the supplied verifier. It is not a production readiness claim.

## Release posture

G0–G5 remain blocked, G6 is not started, G7 is not started, and overall release remains **NO-GO**. Governance decisions and production verifier/storage choices remain open.
