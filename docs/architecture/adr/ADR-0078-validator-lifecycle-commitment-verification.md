# ADR-0078 — Validator Lifecycle Commitment Verification API

- **Status:** Proposed
- **Date:** 2026-11-08
- **Related phase:** 20G2D22
- **Supersedes:** None

## Context

Phase 20G2D21 introduced a deterministic commitment over a fully audited lifecycle history. Consumers need a typed operation to compare a recomputed digest with an expected value without duplicating audit/commitment logic.

## Decision

Add `verify_lifecycle_commitment` as a thin, read-only verification boundary. It delegates digest generation to `LifecycleLedger::commitment`, propagates ledger failures through a typed error, and reports digest mismatch distinctly.

## Consequences

- Callers receive one fail-closed operation for audit, recomputation and comparison.
- The ledger is not mutated.
- The expected digest remains untrusted input; this API does not authenticate its source or freshness.
- Durable monotonic checkpointing and external authentication remain separate future work.

## Rejected shortcuts

- Comparing an expected digest without rerunning the ledger audit.
- Treating a matching unkeyed digest as authenticated provenance.
- Introducing persistence or signer authority in this phase.
