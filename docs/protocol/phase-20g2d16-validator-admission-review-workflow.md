# Phase 20G2D16 — validator admission review and decision workflow

**Status:** Proposed workflow; not governance-ratified or implemented.  
**Base main:** `f8cb64fb6d4b2d1a518c71554fae118517b1d1d7`.  
**Related:** Phases 20G2D13–20G2D15; ADR-0069, ADR-0070, ADR-0071.  
**Release impact:** G2/G4 remain BLOCKED; production remains NO-GO.

## Objective

Define a review lifecycle for the per-operator evidence contract without appointing an authority, granting discretion to bypass technical requirements, or changing validator-set state. The workflow separates evidence intake, technical assessment, governance decision, and deterministic activation.

This phase is documentation-only. It does not create an admission process that is currently in force.

## 1. Non-negotiable separation of concerns

The following are distinct states and must never be treated as synonyms:

1. **Submitted:** an operator has supplied an evidence package.
2. **Intake-validated:** package envelope, scope and required inventory can be parsed and checked for completeness.
3. **Technically assessed:** scoped evidence has been evaluated against published baseline requirements.
4. **Decision-ready:** conflicts, material gaps and reviewer findings have been recorded for an authorized decision-maker.
5. **Admission decision:** an authorized governance process has issued an approval, rejection, deferral or remediation disposition.
6. **Activation-eligible:** all protocol-defined prerequisites for a validator-set transition are satisfied.
7. **Active:** the validator participates only after the deterministic protocol transition takes effect.
8. **Suspended, exiting or removed:** a distinct lifecycle state reached only through the applicable authorized and protocol-defined process.

A complete package is not proof of eligibility. Technical eligibility is not admission approval. Admission approval is not validator-set activation. No off-chain reviewer, operator or governance actor may directly edit consensus membership or bypass deterministic transition rules.

## 2. Proposed review lifecycle

### Stage A — submission and intake

- Assign a stable package and revision identifier.
- Verify contract version, operator identity claim, operator class, validator scope and submission purpose.
- Check that required evidence categories are present or explicitly marked not applicable with rationale.
- Verify referenced artifact availability and declared integrity metadata.
- Record submission time, intake actor, package digest and any intake exceptions.
- Reject malformed or ambiguous scope for correction; do not silently reinterpret it.

Intake checks establish package processability, not the truth of evidence claims.

### Stage B — reviewer assignment and independence

- Assign reviewers only under a future ratified reviewer appointment policy.
- Require conflict-of-interest disclosure before access to restricted evidence.
- A reviewer with a material financial, organizational, operational, personal or common-control conflict must recuse from the affected assessment.
- Record assignment, access, recusal, replacement and rationale in an auditable review log.
- Separate evidence production by the operator from independent verification wherever practical.
- No operator may select, direct or privately compensate the reviewer responsible for its independent assurance.

The required number of reviewers, appointment authority, independence test, quorum and tie-breaking method remain undecided. This document does not create a reviewer body.

### Stage C — evidence assessment

Assess each evidence item against the exact requirement and declared scope. Record:

- requirement and contract version;
- item/package identifier and digest;
- verification method and reproducibility;
- evidence provenance and freshness;
- result: PASS, FAIL, INCONCLUSIVE, NOT APPLICABLE or NOT SUPPLIED;
- limitations, contradictions and residual risk;
- reviewer identity/role and timestamp.

Reviewers must not infer a pass from package completeness, an operator attestation alone, a digest alone, or a test result outside its declared version/configuration/coverage.

Safety-critical uncertainty remains INCONCLUSIVE. A missing, stale, contradictory or unverifiable critical item cannot be waived by an individual reviewer. Any future exception mechanism must be separately ratified, bounded, transparent and incapable of waiving cryptographic, anti-equivocation or anti-rollback safety requirements.

### Stage D — findings and remediation

- Consolidate item-level findings without erasing dissenting assessments.
- Classify each finding by requirement, severity, affected validator scope and evidence.
- Give the operator a documented opportunity to correct factual errors or supply missing evidence under a future ratified response window.
- Preserve the original submission, revised package, reviewer findings and change history.
- Reassess only affected claims plus any dependencies invalidated by the change.
- Do not allow remediation to overwrite or conceal a prior failure.

No remediation deadline, grace period, severity taxonomy or interim operating permission is selected here.

### Stage E — decision record

A future authorized decision record should include:

- decision identifier and governing authority;
- operator and exact validator scope;
- package revision and evidence digest;
- contract and baseline versions;
- reviewer assignments, conflict disclosures and recusals;
- item-level outcomes and unresolved findings;
- decision: approve, reject, defer, require remediation, or accept withdrawal;
- rationale, conditions, effective period and review triggers;
- dissenting opinions and appeal information;
- explicit statement that the decision does not itself activate a validator.

Decision-makers may not convert a failed mandatory technical requirement into a pass by policy preference. Any legally or governance-permitted exception must identify its authority, scope, compensating controls, expiration and public rationale, and must remain outside prohibited cryptographic safety requirements.

### Stage F — appeal and dispute

The future framework should provide a reviewable path to challenge factual errors, inconsistent application of published criteria, undisclosed conflicts or procedural defects.

- Preserve the original decision and evidence snapshot.
- Identify the grounds, challenged findings and requested remedy.
- Assign appeal reviewers who did not make or materially influence the original decision, subject to a ratified independence policy.
- Record submissions, evidence considered, recusals, reasoning and outcome.
- Distinguish correction of the record, remand for reassessment and reversal of a governance decision.
- An appeal does not itself authorize signing, activation, continued operation or restoration of consensus power.

Appeal authority, filing window, stay effect, review standard and finality remain undecided. No automatic stay or automatic continuation of validator activity is implied.

## 3. Lifecycle controls for suspension, exit and replacement

A future policy must specify objective triggers, authority, notice, evidence preservation, response rights and protocol transition effects for suspension, voluntary exit, removal and replacement.

- An administrative finding must not directly mutate the validator set.
- Any emergency safety action must be narrowly bounded, attributable, reviewable and reconciled with deterministic protocol transitions.
- Signing authority and key custody remain with the operator; admission review does not grant access to keys or signer systems.
- A suspension or removal decision must identify the exact validator identity and effective protocol transition, not rely on an ambiguous operator-wide label.
- Exit and replacement must preserve signer safety, anti-equivocation history and recovery evidence. A replacement identity is not a continuation of the old identity unless protocol rules explicitly define it.

No emergency authority, slashing rule, suspension threshold, transition timing or replacement mechanism is established in this phase.

## 4. Public record, restricted evidence and auditability

The public decision summary should disclose, at minimum, the decision identifier, operator/validator scope at an approved granularity, contract version, decision outcome, material rationale, conditions, effective status and appeal status.

Restricted evidence access must be purpose-limited, logged and reviewable. Public redaction must not conceal a material failure, conflict, common-control relationship or limitation necessary to understand the decision. Private keys, seed phrases and reusable signer credentials are never admissible evidence.

The audit trail should make it possible to reconstruct which package revision, requirements, reviewers and findings informed a decision. This phase does not choose a registry, storage system, signature format, retention period or public transparency technology.

## 5. Required invariants and negative tests

Any future implementation must demonstrate at least:

1. Package completeness cannot directly produce an admission approval.
2. Technical PASS cannot directly activate a validator.
3. A reviewer conflict prevents that reviewer from issuing the affected independent assessment.
4. Reviewer recusal and replacement are preserved in the audit record.
5. Stale, missing, contradictory or unverifiable safety-critical evidence remains non-passing.
6. A revised package cannot erase prior evidence or findings.
7. A decision references an immutable package revision/digest and exact validator scope.
8. An appeal cannot silently rewrite the original decision or bypass protocol transitions.
9. Suspension/removal decisions cannot mutate consensus membership outside deterministic protocol rules.
10. Review tooling has no signer-key access and cannot invoke signing operations.
11. Protocol-operated and independent operators are assessed against equivalent published baseline criteria.
12. A governance exception cannot waive cryptographic, anti-equivocation or anti-rollback safety requirements.

Tests must include adversarial cases: forged or mismatched evidence references, stale evidence, reviewer collusion/conflict, package substitution, scope expansion, hidden common control, appeal reviewer conflict, replayed decisions and attempted direct validator-set mutation.

## 6. Open decisions — no defaults inferred

The following require explicit governance and technical decisions:

- admission authority and separation from protocol implementation/operators;
- reviewer appointment, independence, number, quorum and tie-break process;
- baseline requirement versioning and permissible exception boundaries;
- remediation periods, severity taxonomy and interim status;
- appeal authority, grounds, deadlines, standard and stay effect;
- public disclosure granularity and restricted-evidence access policy;
- suspension, emergency action, exit, removal and replacement authority;
- deterministic validator-set transition semantics and activation delay;
- decision/evidence retention, integrity and audit technology;
- implementation ownership, threat model and independent security review.

## 7. Acceptance gates

This phase may be considered a complete proposed workflow only when the documentation and negative-test requirements are reviewed. It must not be marked governance-ratified or production-ready.

Before ADR-0072 can be Accepted or any admission workflow implemented, require:

- formal ratification of authority, reviewer independence, conflicts, privacy, appeals and decision boundaries;
- objective, versioned eligibility criteria and bounded exception rules;
- deterministic and tested activation, suspension, exit, removal and replacement transitions;
- production signer conformance and complete native signing-byte/key-use interception proof;
- durable reservation-before-key-call, anti-rollback and recovery proof;
- independent security review of the workflow, evidence access and audit trail.

**Current gate:** ADR-0072 Proposed; governance ratification absent; G2/G4 BLOCKED; production NO-GO.
