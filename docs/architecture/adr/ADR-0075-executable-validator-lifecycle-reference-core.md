# ADR-0075: Executable validator lifecycle reference core

- **Status:** Proposed
- **Date:** 2026-11-08
- **Phase:** 20G2D19
- **Related:** ADR-0067, ADR-0068, ADR-0073, ADR-0074

## Context

ADR-0074 defines lifecycle transition conformance but deliberately leaves policy, authority, persistence, canonical encoding and protocol proof verification open. A purely documentary contract is difficult to exercise against stale revisions, retries, state-domain confusion and membership transition ordering.

## Decision

Add an executable, policy-neutral Rust reference state machine in `apex-consensus::validator_lifecycle`. It keeps application, eligibility, governance disposition and consensus membership as separate state domains; binds records to chain/protocol/operator/validator scope; rejects stale revisions and request-ID conflicts; tracks evidence/policy revisions; and retains append-only in-memory history.

Activation and exit confirmation require an injected `ProtocolTransitionVerifier`. No production verifier is provided. Administrative suspension/revocation changes governance disposition only and does not rewrite consensus membership.

## Consequences

- Transition guards and state-domain separation can be exercised by automated tests.
- Duplicate exact requests are idempotent within the in-memory process; conflicting reuse of a request ID fails closed.
- The implementation remains a reference model: no durable storage, authenticated actor/evidence model, distributed ordering, canonical wire format, production proof verifier, live validator-set mutation, signer or key integration.
- Governance decisions and protocol transition semantics remain unratified and cannot be inferred from this code.

## Acceptance evidence

Accept only after independent review of the transition matrix and implementation, explicit governance ratification, canonical envelope/proof specification, durable transaction/recovery design, production consensus proof verification, end-to-end validator-set and signer interlock tests, and independent security review.

**Current gate:** Proposed. G2/G4/G5 BLOCKED. Production NO-GO.
