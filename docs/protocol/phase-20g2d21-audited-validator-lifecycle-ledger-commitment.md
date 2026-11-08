# Phase 20G2D21 — Audited Validator Lifecycle Ledger Commitment

## Objective

Add a deterministic, domain-separated SHA-256 commitment over the validator lifecycle reference ledger's ordered event history.

## Implemented contract

- LifecycleLedger::commitment(verifier) first runs the full Phase 20G2D20 audit and fails closed if any history/index invariant or proof verification fails.
- Encode scopes in BTreeMap order and records in append order.
- Encode every lifecycle event and resulting lifecycle state with explicit, versioned one-byte discriminants.
- Encode integers as big-endian fixed-width values.
- Prefix variable-length protocol proof bytes with a big-endian u64 length.
- Hash under the domain tag APEX/validator-lifecycle-ledger-commitment/v1.
- Return a 32-byte SHA-256 digest.
- Do not mutate the ledger.

## Tests

- Identical audited histories produce identical commitments.
- A valid additional transition changes the commitment.
- A ledger with a divergent request index cannot produce a commitment.

## Encoding and security boundaries

This is a versioned, local reference encoding used only to compute a deterministic commitment. It is not a canonical protocol wire format, general-purpose serialization contract, authenticated checkpoint, signature, trusted timestamp, anti-rollback anchor, or proof of history provenance. A party able to rewrite the in-memory history can recompute the digest. The supplied protocol verifier must remain deterministic and trustworthy.

The commitment includes event history and recorded resulting states. Derived indexes are not independently hashed because the audit verifies their exact equality with replayed history before hashing.

## Release posture

The lifecycle ledger remains in-memory and single-process. Durable persistence, crash recovery, cross-process ordering, authenticated external anchoring, production proof verification, governance authority, signer integration and live consensus validator-set mutation remain out of scope. G0–G5 remain blocked; G6/G7 remain not started; overall release remains **NO-GO**.
