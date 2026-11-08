# ADR-0071: Per-operator validator admission evidence and conformance contract

- **Status:** Proposed
- **Date:** 2026-11-08
- **Phase:** 20G2D15
- **Related:** ADR-0067, ADR-0068, ADR-0069, ADR-0070

## Context

ADR-0070 proposes a hybrid validator admission model based on objective eligibility requirements with bounded governance oversight. That model needs a consistent way for protocol/ecosystem-operated and independent operators to demonstrate conformance.

Without a shared evidence contract, admission reviews risk accepting incomparable claims, stale artifacts, unbounded disclosure or undocumented reviewer discretion. At the same time, defining an evidence package must not accidentally select a registry schema, cryptographic format, governance authority or validator-set transition mechanism.

## Proposed decision

Adopt a versioned, per-operator evidence package as the proposed common contract for demonstrating validator admission baseline conformance.

The package should:
1. identify the operator, operator class, validator scope, contract version and submission purpose;
2. enumerate scoped claims with provenance, method, observation time, freshness, limitations, integrity reference and disclosure classification;
3. cover operator accountability, validator/protocol conformance, signer safety/key lifecycle, operational readiness, fault-domain concentration and governance conduct;
4. distinguish pass, fail, inconclusive, not applicable and not supplied evidence outcomes;
5. distinguish package completeness from eligibility, admission approval and consensus activation;
6. preserve traceable revisions and disclose material evidence limitations;
7. apply equivalent requirements to protocol-operated and independent operators;
8. prohibit private-key disclosure or transfer of signing authority as an evidence requirement.

A digest or attestation does not by itself establish truth, provenance, completeness, freshness or correctness. Evidence must be assessed within its declared scope and limitations. Safety-critical uncertainty remains inconclusive and cannot be discretionarily represented as a pass.

## Authority and implementation boundaries

This ADR:
- does not ratify an admission authority or governance body;
- does not establish an admission registry, wire format, database schema, package-signature format or evidence repository;
- does not select numerical thresholds, freshness periods, retention periods or diversity targets;
- does not create exceptions, sanctions, slashing, emergency powers or appeals;
- does not activate or remove validators;
- does not change validator-set, consensus, signer, key, WAL or runtime behavior.

A complete evidence package is only an input to a future authorized review. Admission approval, if later ratified, remains distinct from activation. Consensus participation changes only through deterministic protocol-defined validator-set transition semantics.

No reviewer receives private-key custody, signer-store access or signing authority.

## Consequences

- Operators need a repeatable, privacy-aware way to assemble and refresh conformance evidence.
- Reviewers need explicit evidence scope, freshness and limitation metadata.
- Public summaries must not expose secrets or exploitable topology, but redaction cannot conceal material findings.
- Technical test results must be scoped to exact software, configuration and coverage.
- Production signer safety and recovery remain separate proof obligations.
- G2/G4 remain BLOCKED; production remains NO-GO.

## Acceptance evidence

Change this ADR to **Accepted** only after:
- governance ratifies the authority, review, privacy and appeal boundaries;
- evidence requirements and disclosure rules are approved;
- the package's integrity, provenance and freshness model is specified;
- reviewer independence and conflict handling are approved;
- deterministic validator-set activation, suspension, exit and replacement semantics are specified and tested;
- production signer conformance, exact native signing-byte coverage, reservation-before-key-call, anti-rollback and recovery are independently demonstrated;
- security review confirms the evidence process cannot obtain operator signing authority or bypass protocol safety controls.
