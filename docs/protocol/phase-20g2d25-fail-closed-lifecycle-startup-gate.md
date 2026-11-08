# Phase 20G2D25 — Fail-Closed Lifecycle Startup Witness Gate

## Objective

Make the lifecycle rollback-witness check an explicit startup gate over every configured network. A node must not proceed when a local checkpoint is missing, the witness is unavailable, or the two heads diverge.

## Implementation

- Add `verify_lifecycle_startup` to `apex-lifecycle-redb`.
- Require a non-empty configured network set.
- Reject zero and duplicate network IDs before verification.
- For every network, load the durable local head and independently witnessed head.
- Require both heads to be initialized and exactly equal.
- Return network-scoped typed errors for local-store failure, witness failure, uninitialized state, and head mismatch.
- Perform no mutation, auto-repair, genesis initialization, or recovery.

## Startup contract

The caller must invoke this gate before starting consensus, signing, networking, or any service that can act on lifecycle state. The current `apex-node` binary remains a placeholder and does not yet run a node startup sequence; therefore this phase supplies and tests the gate, but does not claim a live node is protected by it.

The witness trait remains an integration boundary. No production witness provider, independently administered service, TPM/HSM binding, authenticated transport, credential lifecycle, or deployment configuration is selected or implemented. The caller must ensure the implementation is in a separate trust domain; a second file or database on the same host is not sufficient evidence of independence.

## Security behavior

- Local storage unavailable: abort startup.
- Witness unavailable: abort startup.
- Local and witness heads differ: abort startup.
- Either side uninitialized: abort startup. Genesis/bootstrap must be performed through a separately authorized procedure.
- Duplicate or zero network ID: reject configuration.
- No automatic reconciliation or rollback acceptance.

## Validation

Tests cover empty configuration, zero and duplicate IDs, multiple matching networks, mismatch, witness outage, and uninitialized state. CI must pass on the exact PR head and after merge.

## Release posture

This is a fail-closed startup verification boundary, not a production independent witness provider or anti-rollback deployment. G0–G5 remain blocked; G6/G7 remain not started; overall release remains **NO-GO**.
