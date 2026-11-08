# Phase 20G2D15 — per-operator validator admission evidence and conformance contract

**Status:** Proposed evidence contract; not governance-ratified or implemented.  
**Base main:** `7ea69fd5d7e20ed2bfeb016b9e8fb8dc8c190251`.  
**Related:** Phase 20G2D13, Phase 20G2D14, ADR-0069, ADR-0070.  
**Release impact:** G2/G4 remain BLOCKED; production remains NO-GO.

## Objective

Define the minimum evidence package an individual validator operator must produce to demonstrate conformance with the proposed hybrid admission baseline. Establish a common, auditable review vocabulary for protocol-operated and independent operators without selecting an admission authority, numeric threshold, registry schema, signing backend or governance procedure.

This is a documentation-level contract. It does not authorize admission, activate a validator, create an enforceable governance obligation, or establish that any existing operator is compliant.

## 1. Contract principles

1. **One operator, one accountable evidence package:** each package identifies the operator and the validator identities within its declared scope.
2. **Equivalent baseline:** protocol/ecosystem-operated and independent operators provide evidence against the same requirements and review criteria.
3. **Evidence, not assertion:** claims must identify their subject, scope, provenance, observation time, applicable software/configuration and verification method.
4. **Reproducibility:** where practical, an independent reviewer can reproduce the check without privileged access to operator secrets.
5. **Least disclosure:** publish a verifiable summary and redact credentials, private keys, personal data, exploitable topology and other sensitive details.
6. **Freshness is explicit:** every item declares its observation time, validity window or refresh trigger; no universal expiry duration is selected here.
7. **No implied approval:** submission, completeness or a passing technical check does not itself grant admission or consensus power.
8. **No key transfer:** evidence collection must not require the operator to disclose private keys, seed phrases, signer credentials or unrestricted administrative access.
9. **Fail closed on critical uncertainty:** missing, stale, contradictory or unverifiable safety-critical evidence cannot be represented as a pass.
10. **Auditable outcomes:** every review result records the contract version, evidence digest, checks performed, findings and disposition.

## 2. Evidence package envelope

A conforming package is a logical, versioned collection. This phase deliberately does not define a wire format, database schema, signature envelope or registry API.

Every package must identify:

| Field | Required meaning |
|---|---|
| Contract identifier/version | Exact admission evidence contract and revision used |
| Package identifier | Stable unique identifier for this submission/revision |
| Operator identifier | Stable identifier for the accountable operator |
| Operator class | Protocol/ecosystem-operated or independent |
| Validator scope | Validator identity/identities covered; no ambiguous wildcard scope |
| Submission purpose | Initial application, periodic refresh, material change, remediation, appeal or exit |
| Created/observed time | UTC timestamp and source/clock assurance where relevant |
| Evidence inventory | Item identifiers, category, status, provenance and digest |
| Software/configuration identity | Reproducible release, build and configuration references |
| Prior package reference | Previous package identifier/digest when this is a revision |
| Disclosure classification | Public summary, reviewer-restricted or operator-confidential |
| Responsible attestation | Authorized operator representative and scope of their assertion |

A package revision must preserve a traceable relationship to the prior revision. Superseded evidence is retained according to a future ratified retention policy; this phase does not select retention periods or storage technology.

## 3. Evidence item contract

Each evidence item must include:

- **Item ID and category:** stable within the package and mapped to a requirement below.
- **Claim:** one narrowly scoped statement being demonstrated.
- **Subject and scope:** operator, validator identity, software release, infrastructure domain or process covered.
- **Method:** test, inspection, independent attestation, operational record or other declared verification method.
- **Provenance:** producing system/person, source reference and chain of custody where applicable.
- **Observed time and freshness:** when the evidence was generated and what invalidates it.
- **Result:** pass, fail, inconclusive, not applicable or not supplied.
- **Limitations:** assumptions, exclusions, known gaps and dependencies.
- **Integrity reference:** content digest or equivalent tamper-evident identifier; cryptographic format remains open.
- **Disclosure treatment:** what can be published and what requires restricted review.

A digest proves byte identity only when the referenced bytes are available and the digest is trusted. It does not prove truth, origin, completeness, freshness or correctness of the claim.

## 4. Required evidence domains

### A. Operator accountability and common control

Provide:
- accountable operator identity and responsible contact channel;
- operator class and organizational ownership/control declaration;
- disclosure of common control with other validator operators where relevant;
- process and evidence for ownership/control changes;
- conflict-of-interest declarations relevant to admission and independence.

Public disclosure must be limited to information needed for accountability and concentration analysis. Identity verification method, legal-personhood threshold and privacy regime remain governance decisions.

### B. Validator identity and protocol conformance

Provide:
- exact validator identity and its relationship to the operator;
- approved protocol/client release identifier and reproducible build provenance;
- configuration profile and protocol-version compatibility evidence;
- results from versioned consensus and validator conformance suites;
- negative tests showing rejection of unsupported or unsafe protocol configurations;
- controlled upgrade, rollback and recovery procedure evidence.

A passing test suite is scoped to its exact version, configuration and test coverage. It is not proof of production correctness or freedom from defects.

### C. Signer safety and key lifecycle

Provide evidence for:
- operator-controlled key custody and access boundaries;
- signer isolation and authorization policy;
- key generation, activation, rotation, retirement and compromise response;
- exact native signing-byte and trusted-context conformance for every signing path;
- durable reservation-before-key-call behavior;
- duplicate/replay handling and conflicting-intent rejection;
- anti-rollback protection and recovery behavior;
- fail-closed treatment of ambiguous key-call outcomes;
- audit coverage for every path capable of invoking the signing key.

Evidence must distinguish design documentation, isolated test evidence, integration evidence and production-observed evidence. A test signer or generic journal test is not production signer proof. APEX's Phase 20G2A preimage and Malachite native signing bytes remain distinct domains and must not be conflated.

No evidence package may include private key material or require the reviewer to exercise signing authority.

### D. Operational readiness and resilience

Provide:
- monitored service and alerting coverage;
- incident contact and escalation process;
- time synchronization and capacity monitoring evidence;
- backup/restore and disaster-recovery exercise records;
- upgrade and security-patching process;
- availability and response measurements with declared measurement window and methodology;
- dependency inventory for hosting, network, DNS, RPC, identity, monitoring and backup domains.

No uptime, response-time, geographic or provider-diversity threshold is selected in this phase.

### E. Fault-domain and concentration disclosure

For each validator in scope, disclose enough to assess correlated failure across:
- common ownership and administrative control;
- signing control and recovery authority;
- hosting provider, account and region;
- network, DNS, RPC and upstream dependencies;
- client implementation, release and configuration;
- monitoring, identity and backup dependencies.

The evidence must state where values are exact, aggregated, independently attested or redacted. Multiple validator identities under one control plane must not be represented as independent fault domains. Numerical diversity targets and public disclosure granularity remain open.

### F. Governance and conduct

Provide:
- acknowledgment of the currently applicable published protocol rules;
- conflict-of-interest and common-control declarations;
- incident notification and evidence-preservation procedures;
- acknowledgment of the applicable review, remediation and appeal process once ratified.

This contract does not invent or activate a governance body, sanction, slashing rule, emergency actor or appeal authority.

## 5. Verification and review outcomes

Reviewers should evaluate each item independently and then assess package-level completeness and contradictions.

Permitted evidence outcomes:
- **PASS:** claim is supported within the declared scope and freshness.
- **FAIL:** evidence contradicts a mandatory requirement or a reproducible check fails.
- **INCONCLUSIVE:** evidence is insufficient, contradictory, stale or not independently verifiable.
- **NOT APPLICABLE:** requirement is demonstrably outside the declared scope, with rationale.
- **NOT SUPPLIED:** required evidence is absent.

Permitted package dispositions:
- **COMPLETE FOR REVIEW:** required evidence inventory is present; this is not admission approval.
- **REMEDIATION REQUIRED:** identified gaps may be corrected under a future ratified process.
- **INELIGIBLE UNDER BASELINE:** a mandatory baseline requirement is not met.
- **CONTESTED:** a finding is subject to a future ratified dispute/appeal process.
- **WITHDRAWN/EXITING:** operator has withdrawn or is documenting safe exit.

No reviewer may convert an inconclusive safety-critical result into a pass by discretion. Any future permitted exception must follow the ratified exception framework, identify compensating controls and expiry, and cannot waive critical cryptographic, anti-equivocation or anti-rollback protections.

## 6. Freshness, change triggers and reassessment

Evidence must be refreshed when its declared validity expires or a material change invalidates the claim. Material changes include:
- operator ownership or common-control changes;
- validator identity, key or signer-policy changes;
- signer software, protocol client, build or critical configuration changes;
- hosting, region, network or administrative-domain changes;
- compromise, security incident or recovery event;
- failed conformance check or material operational outage;
- changes to dependencies that affect fault-domain independence.

The refresh interval, grace period, suspension trigger and evidence retention schedule remain unselected. A stale item must be marked stale, not silently reused.

## 7. Privacy, security and publication boundary

The public record should contain package identity, contract version, covered validator scope, evidence categories, outcome summaries, material limitations and decision rationale once an authorized process exists.

Restricted evidence may include sensitive topology, personal information, incident details and security controls. Access must be purpose-limited, logged and reviewable. No secret or private key material is admissible evidence.

Redaction must not hide a material failure, common-control relationship or limitation needed to evaluate the claim. A public summary should identify that restricted evidence exists and state the nature of the redaction without exposing exploitable details.

No storage, encryption, access-control or retention implementation is selected here.

## 8. Relationship to admission and validator-set transitions

This evidence contract is upstream of, and separate from:
1. governance ratification of the admission authority and procedures;
2. a canonical operator/validator identity model;
3. registry schema, provenance and privacy controls;
4. deterministic validator-set update authorization, ordering and activation;
5. suspension, revocation, rotation, exit and replacement semantics;
6. production signer and recovery assurance.

A package marked COMPLETE FOR REVIEW does not admit a validator. An admission decision does not activate consensus power. Consensus participation changes only through a separately specified, deterministic, protocol-defined validator-set transition with an explicit effective height/epoch.

The admission reviewer has no key custody, signer-store access or ability to produce signatures for an operator.

## 9. Open decisions

- Governance authority and ratification procedure.
- Identity verification and privacy model.
- Cryptographic attestation/digest and package-signature format.
- Evidence repository, access control, retention and audit implementation.
- Numerical reliability, freshness and diversity thresholds.
- Reviewer independence, conflict handling and appeal process.
- Exception authority and emergency review boundaries.
- Deterministic validator-set transition semantics and activation rules.
- Production signer, native-byte coverage, anti-rollback and recovery proof.

No implementation or numeric threshold may be inferred from this proposed contract.

## Exit criteria

- Versioned evidence package envelope and evidence-item requirements documented.
- Operator accountability, conformance, signer safety, operations, diversity and governance evidence domains defined.
- Review outcomes and package dispositions distinguished from admission approval.
- Freshness, change triggers, privacy and publication boundaries specified.
- Equivalent evidence obligations stated for protocol-operated and independent operators.
- No registry, schema, contract, governance authority, validator-set, signer or runtime behavior implemented.
- ADR-0071 remains Proposed; G2/G4 remain BLOCKED; production remains NO-GO.

## Next permitted work

Governance review must ratify the authority, privacy and review boundaries. Engineering follow-on work may separately specify canonical evidence encoding and deterministic validator-set transitions only after the relevant decisions and signer/recovery gates are satisfied.
