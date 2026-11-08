# Phase 20G2D19 — executable validator lifecycle reference core

**Status:** Implementation submitted for CI and review; isolated local state machine only.  
**Parent:** Phase 20G2D18; ADR-0074.  
**Base:** `87376a78d9541cb600cce0d9a7a750624cb4ebcb`.  
**Release impact:** G2/G4/G5 remain BLOCKED; production NO-GO.

## Objective

Turn the Phase 20G2D18 lifecycle transition contract into executable Rust behavior without inventing governance authority, policy thresholds, consensus semantics, persistence, or signer integration.

## Implementation

Adds `crates/apex-consensus/src/validator_lifecycle.rs` and exports it from `apex-consensus`.

The module provides:
- Four independently represented state domains: application, eligibility, governance disposition and consensus membership.
- A scoped lifecycle key binding chain ID, protocol version, operator ID and validator ID.
- Version-checked transition envelopes and per-scope monotonically increasing local revisions.
- Evidence revision and policy revision guards.
- Explicit submission, review, remediation, evidence resubmission, eligibility, approval, denial, appeal and withdrawal transitions.
- Separate activation/exit proposal and protocol-commit observation transitions.
- An injected `ProtocolTransitionVerifier` boundary for activation/exit proof verification; there is no production verifier in this phase.
- Exact-envelope idempotency by request ID; reuse of an ID for a different envelope fails closed.
- Append-only in-memory transition history and scope isolation.
- Tests for domain separation, proof-verification failure, activation/exit, revocation not mutating membership, stale revisions, idempotency, evidence revision monotonicity and history ordering.

## Safety boundaries

This is executable reference behavior, not a production lifecycle service:
- The ledger is in-memory and single-process; no durable storage, crash recovery, distributed concurrency or authenticated audit chain is implemented.
- Request IDs and decision references are opaque identifiers, not cryptographic authentication.
- The module does not select or enforce governance actors, quorum, thresholds, deadlines, appeals authority or emergency powers.
- The injected protocol verifier is an interface only; test verifier behavior is test-only and is not production proof verification.
- The module does not mutate the live validator set, voting power, Malachite engine, WAL, signer policy or key material.
- Membership changes in this reference ledger occur only after the injected verifier accepts the supplied protocol proof. Governance revocation/suspension never directly rewrites consensus membership.
- No wire format, canonical codec, database schema, API or production integration is selected.

## Validation

The phase is submitted for the repository's existing Rust CI:
- workspace formatting, Clippy and unit tests;
- isolated Malachite v0.8.2 adapter and upstream test suite;
- four-process adversarial harness.

A green CI result validates this implementation only. It does not ratify ADR-0074, establish consensus correctness, or satisfy G2/G4/G5.

## Follow-on gates

Before production use: ratify governance decisions; define canonical transition encoding and authoritative ordering; add durable transactional storage and restart/replay tests; provide a production protocol proof verifier bound to the selected consensus engine; prove complete validator-set transition and signer interlocks; perform independent security review.

**Decision:** Experimental code only. ADR-0075 Proposed. ADR-0074 remains Proposed. G2/G4/G5 BLOCKED. Production NO-GO.
