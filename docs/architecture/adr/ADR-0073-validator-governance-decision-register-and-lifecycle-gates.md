# ADR-0073: Validator governance decision register and deterministic lifecycle gates

- **Status:** Proposed
- **Date:** 2026-11-08
- **Phase:** 20G2D17
- **Related:** ADR-0067, ADR-0068, ADR-0069, ADR-0070, ADR-0071, ADR-0072

## Context

The hybrid validator model and evidence/review workflow define a direction but intentionally leave constitutional authority, reviewer appointment, eligibility criteria, appeals, exceptions, suspension, exit, removal and validator-set transition semantics unresolved.

If implementation proceeds without explicit decisions, code or operational practice may silently establish governance powers, create ambiguous lifecycle states, or allow off-chain admission decisions to be mistaken for consensus membership. This is particularly unsafe while production signer ownership, exact native signing-byte interception, durable reservation, anti-rollback and recovery remain unproven.

## Proposed decision

Adopt a versioned governance decision register and an abstract validator lifecycle contract as mandatory prerequisites to implementation.

The decision register identifies each unresolved policy question, its minimum ratification record and the consequence of leaving it open. The lifecycle vocabulary distinguishes application/evidence states, governance decisions, pending protocol transitions and actual consensus membership.

The following separation is mandatory:

1. evidence submission and package completeness;
2. technical eligibility;
3. authorized admission decision;
4. activation eligibility and pending transition;
5. active consensus membership after deterministic protocol transition;
6. suspension, exit, removal and replacement decisions versus their actual consensus effects.

No admission, suspension, revocation or exit decision may directly mutate consensus membership. A validator becomes active or inactive only through deterministic protocol-defined validator-set transitions. Review and governance systems receive no signer-key custody or signing authority.

Safety-critical signer requirements—including exact native signing-byte coverage, durable reservation before key invocation, anti-equivocation, anti-rollback and recovery—cannot be waived by governance exceptions.

## Decision boundaries

This ADR does not:
- appoint or select any governance authority, reviewer body or appeal authority;
- select voting rules, quorum, reviewer count, deadlines, thresholds or emergency powers;
- define a complete executable validator-set state machine or height/epoch activation algorithm;
- select a registry, contract, schema, API, cryptographic format, database or retention technology;
- authorize implementation or alter consensus, signer, key, WAL or runtime behavior.

All decision-register items remain OPEN until ratified by the authority and procedure established in the governing constitutional framework. A merged ADR or CI pass is not ratification.

## Consequences

- Governance ambiguity is explicit and cannot be hidden in implementation defaults.
- Future policy changes can be versioned, audited and tested against protocol invariants.
- Operators and reviewers have a clear separation between evidence, eligibility and approval.
- Consensus membership remains a deterministic protocol fact, not an administrative label.
- Implementation remains blocked until governance decisions, transition semantics and production signer/recovery evidence are independently established.

## Acceptance evidence

Change this ADR to **Accepted** only after:
- the constitutional authority and decision procedure are established and ratified;
- all blocking governance decisions have adopted, versioned records;
- a complete validator-set lifecycle transition specification is approved;
- deterministic implementation and adversarial/replay tests verify every transition and failure path;
- production signer integration proves exact native bytes, complete key-use interception and durable reservation-before-key-call;
- anti-rollback and recovery are demonstrated under crash, replay and rollback scenarios;
- independent security review confirms governance/review tooling cannot access signing authority or bypass consensus safety.

**Current gate:** Proposed only. Governance decisions OPEN. G2/G4/G5 BLOCKED. Production NO-GO.
