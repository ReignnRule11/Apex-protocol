# ADR-0072: Validator admission review and decision workflow

- **Status:** Proposed
- **Date:** 2026-11-08
- **Phase:** 20G2D16
- **Related:** ADR-0067, ADR-0068, ADR-0069, ADR-0070, ADR-0071

## Context

ADR-0070 proposes a hybrid validator admission model. ADR-0071 defines a per-operator evidence contract. Those proposals require a review lifecycle that preserves reviewer independence, evidence provenance, procedural fairness and a strict boundary between off-chain decisions and consensus validator-set transitions.

An incomplete workflow could let package completeness be mistaken for approval, permit conflicted reviewers to certify operators, allow safety-critical uncertainty to be waived, or let an administrative decision bypass deterministic protocol transitions.

## Proposed decision

Adopt the following lifecycle as the proposed review model:

1. submission and intake validation;
2. reviewer assignment and conflict-of-interest screening;
3. scoped evidence assessment;
4. findings and remediation;
5. authorized decision and immutable decision record;
6. dispute/appeal;
7. separately governed activation, suspension, exit, removal or replacement through deterministic protocol transitions.

The model requires equivalent published baseline criteria for protocol-operated and independent operators. Reviewers must record item-level results, evidence scope, provenance, freshness, limitations and conflicts. Safety-critical missing, stale, contradictory or unverifiable evidence remains non-passing.

The workflow explicitly separates:
- package submission from package completeness;
- package completeness from technical eligibility;
- technical eligibility from an admission decision;
- an admission decision from activation;
- administrative suspension/removal decisions from consensus validator-set mutation.

No reviewer or governance actor receives signer-key access or signing authority. No decision may directly modify consensus membership outside protocol-defined deterministic transitions.

A future appeal process should preserve the original decision and evidence snapshot, use reviewers independent of the original decision, and publish a reasoned outcome. This proposal does not imply an automatic stay or continued validator activity during an appeal.

## Authority and implementation boundaries

This ADR:
- does not appoint an admission authority, reviewer body or appeal authority;
- does not choose reviewer count, quorum, tie-break, conflict test or appointment process;
- does not select remediation deadlines, appeal windows, stay effects or severity taxonomy;
- does not create emergency powers, sanctions, slashing or exceptions;
- does not choose public disclosure granularity, evidence repository, registry, cryptographic format or retention period;
- does not define validator-set activation delay or suspension/removal/exit/replacement mechanics;
- does not change code, governance contracts, validator-set, consensus, signer, key, WAL, schema or runtime behavior.

Any future exception mechanism must be explicitly ratified, bounded, auditable and unable to waive cryptographic, anti-equivocation or anti-rollback safety requirements.

## Consequences

- Operators receive a predictable, evidence-based review lifecycle.
- Reviewers must disclose conflicts and preserve independent assessment.
- Decision records become traceable to exact evidence revisions and validator scope.
- Appeals and remediation can correct errors without erasing prior records.
- Implementation must prevent administrative workflows from bypassing deterministic consensus transitions.
- Governance ratification and production signer/recovery proof remain prerequisites; G2/G4 remain BLOCKED and production remains NO-GO.

## Acceptance evidence

Change this ADR to **Accepted** only after:
- governance ratifies the admission authority, reviewer independence, conflict handling, privacy, appeal and decision boundaries;
- objective eligibility criteria and permissible exception limits are approved;
- deterministic activation, suspension, exit, removal and replacement semantics are specified and tested;
- decision and evidence integrity, provenance, freshness, access control and audit requirements are specified;
- production signer conformance, exact native signing-byte coverage, reservation-before-key-call, anti-rollback and recovery are independently demonstrated;
- independent security review confirms the workflow cannot obtain signing authority or bypass protocol safety controls.
