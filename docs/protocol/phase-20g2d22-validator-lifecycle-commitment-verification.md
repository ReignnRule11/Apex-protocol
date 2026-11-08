# Phase 20G2D22 — Validator Lifecycle Commitment Verification

## Objective

Provide a fail-closed verification API that recomputes the Phase 20G2D21 audited lifecycle commitment and compares it with a caller-supplied expected digest.

## Contract

- `verify_lifecycle_commitment` recomputes the digest through `LifecycleLedger::commitment`, which performs the full ledger audit first.
- Returns success only when all audit/proof checks pass and the recomputed 32-byte digest exactly matches the expected digest.
- Preserves ledger audit/commitment failures as a typed `Ledger(LifecycleError)` result.
- Returns a distinct typed `Mismatch` when the expected digest differs.
- Does not mutate ledger state.

## Tests

- Exact expected commitment is accepted.
- A one-bit expected-digest mutation is rejected.
- Empty-ledger commitment can be recomputed and verified.

## Security boundary

The expected digest is caller-supplied and is not authenticated by this API. The API does not establish publisher identity, trusted freshness, monotonicity, anti-rollback, durable storage, or provenance. An attacker able to replace both the history and expected digest can still produce a matching pair. Production use requires a separately authenticated and monotonic checkpoint source.

## Release posture

This remains a local, in-memory reference facility. It does not introduce persistence, distributed ordering, signer integration, governance authority, production proof verification, or live consensus validator-set mutation. G0–G5 remain blocked; G6/G7 remain not started; overall release remains **NO-GO**.
