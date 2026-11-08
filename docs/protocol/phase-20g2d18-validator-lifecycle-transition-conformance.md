# Phase 20G2D18 — validator lifecycle transition conformance contract

**Status:** Proposed, documentation-only; not ratified or implemented.  
**Parent:** Phase 20G2D17; ADR-0073.  
**Base:** `efa8f18997c1d54bb6f389616f9765cf6f253836`.  
**Release impact:** G2/G4/G5 remain BLOCKED; production NO-GO.

## 1. Objective

Turn the abstract lifecycle vocabulary from Phase 20G2D17 into a reviewable transition-conformance contract. Define the information each transition must bind, its guard categories, observable result, audit obligations, retry behavior and failure semantics—without selecting governance authorities, thresholds, timing, storage, APIs or consensus algorithms.

This is a specification boundary, not an executable state machine. No runtime or consensus behavior is changed or authorized.

## 2. State domains must remain separate

A conforming implementation MUST model at least four logically distinct domains:

1. **Application:** submitted, under review, remediation required, rejected or withdrawn.
2. **Eligibility:** unknown, eligible or ineligible against a specific versioned baseline and evidence revision.
3. **Governance disposition:** undecided, approved, denied, appealed, suspended or revoked under a ratified policy.
4. **Consensus membership:** inactive, activation pending, active, exit/removal pending or inactive after a committed protocol transition.

One domain MUST NOT be inferred from another. In particular, evidence completeness is not eligibility; eligibility is not admission approval; approval is not activation; a governance decision is not a consensus transition.

## 3. Transition envelope

Every proposed lifecycle transition MUST be represented by a canonical, versioned envelope. The exact encoding remains an open ADR decision. Its semantic fields MUST include:

- chain identifier and protocol version;
- operator identity and validator identity;
- transition type and source/target state;
- policy version and baseline version, where applicable;
- evidence-package identity, revision and integrity reference, where applicable;
- decision-record identity and approval proof, where applicable;
- prior lifecycle record/version to prevent stale updates;
- protocol transition reference and committed result, where applicable;
- actor/principal and authorization context;
- reason code, bounded public rationale and restricted-evidence reference;
- creation/observation metadata sufficient for audit, without treating wall-clock time as consensus ordering.

The envelope MUST bind the exact identity and scope. Unknown, omitted, conflicting or non-canonical safety-relevant fields MUST fail closed. A transition for one chain, protocol, operator or validator MUST NOT be replayable in another context.

## 4. Transition conformance matrix

The table specifies required guard categories. The authority and exact policy predicates remain unresolved until ratified.

| Transition | Required guards | Durable/auditable result | Failure behavior |
|---|---|---|---|
| UNREGISTERED → SUBMITTED | Authenticated submitter; unique scoped identity; package integrity and provenance | Immutable submission reference and initial revision | Reject malformed/duplicate identity; preserve prior record |
| SUBMITTED → UNDER_REVIEW | Intake acceptance; reviewer assignment under conflict policy | Assignment and conflict-screening record | No review state if assignment evidence is absent |
| UNDER_REVIEW → REMEDIATION_REQUIRED | Finding references criterion, evidence revision and rationale | Append-only finding and requested evidence changes | Do not overwrite submitted evidence |
| REMEDIATION_REQUIRED → UNDER_REVIEW | New evidence revision linked to prior package and findings | New review cycle linked to prior cycle | Reject detached or substituted evidence |
| UNDER_REVIEW → ELIGIBLE | All mandatory baseline checks pass; independent assessment complete | Eligibility result bound to baseline and evidence revision | Missing, stale, contradictory or inconclusive evidence cannot pass |
| UNDER_REVIEW → REJECTED | Authorized decision procedure; recorded grounds and evidence | Immutable disposition record | Invalid authority/procedure leaves case undecided |
| Any nonterminal application → WITHDRAWN | Authenticated applicant authority; withdrawal scope | Withdrawal record; review history retained | No deletion or rewriting of history |
| ELIGIBLE → ADMISSION_APPROVED | Ratified authority and procedure; exact identity/scope; valid decision record | Approval bound to policy, evidence and identity | No implicit approval from timeout, package completion or UI action |
| ELIGIBLE → admission denied | Ratified authority and procedure; recorded grounds | Immutable denial record | No consensus membership effect |
| ADMISSION_APPROVED → ACTIVATION_PENDING | Ratified protocol prerequisites; exact transition proposal; signer readiness gate | Pending transition reference, not active membership | No signing or activation based only on approval |
| ACTIVATION_PENDING → ACTIVE | Valid deterministic consensus transition committed under protocol rules | Committed transition identity and resulting validator-set evidence | Stale, conflicting or unverifiable proof fails closed |
| ACTIVE → SUSPENSION_PENDING | Authorized, scoped suspension decision and defined protocol effect | Pending decision/transition link | Active membership remains a consensus fact until transition commits |
| ACTIVE → EXIT_PENDING | Authenticated voluntary exit or authorized removal request | Pending exit reference | No immediate removal or identity reuse |
| SUSPENSION_PENDING / EXIT_PENDING → INACTIVE | Deterministic protocol transition committed | Committed transition identity and resulting set evidence | No administrative assertion may substitute for commit proof |
| ADMISSION_APPROVED → REVOKED | Ratified revocation process, scope and reason | Revocation record; protocol effect tracked separately | Approval record remains auditable; no silent erasure |
| INACTIVE → replacement application | New identity and independently validated evidence | New lifecycle record linked to predecessor only for audit | No inherited eligibility, approval, key or signer history |

A state transition not represented by the ratified transition table MUST be rejected. This matrix does not define the complete protocol state machine, height/epoch boundaries, activation delays or voting-power update algorithm.

## 5. Ordering, concurrency and idempotency

A future implementation MUST establish a single authoritative ordering mechanism for each state domain. Wall-clock timestamps, API arrival order, database insertion order and dashboard refresh order MUST NOT be treated as consensus ordering.

- Every accepted transition MUST reference the exact prior record/version it advances.
- Two incompatible transitions from the same prior version MUST NOT both commit.
- Repeated submission of the same transition envelope MUST have a deterministic idempotent result or a documented explicit rejection.
- A semantically different envelope reusing the same transition identity MUST fail closed.
- Delayed, reordered, stale-policy or stale-evidence transitions MUST be rejected or routed through a newly authorized review cycle.
- Concurrent review and governance decisions MUST preserve conflict-of-interest and separation-of-duty controls.
- Retries MUST NOT create duplicate active membership, duplicate exit effects or duplicate signer authority.

The concurrency primitive, durable transaction boundary and distributed coordination mechanism remain unselected.

## 6. Consensus boundary and proof of effect

Administrative systems may submit transition proposals and verify public protocol results. They MUST NOT directly write validator-set membership, voting power, consensus state or signer enablement.

A transition is effective only when a protocol-verifiable committed result establishes it. The result MUST be bound to chain, protocol, validator identity, transition type and the protocol-defined transition position. The exact proof format and verification authority require a future design decision.

The following are not proof of effect:
- admission approval or revocation letter;
- governance vote or event without protocol transition;
- API success response or database status;
- operator assertion, monitoring alert or dashboard label;
- local wall-clock deadline or an off-chain scheduled task.

A pending transition MUST remain visibly pending until independently verifiable protocol evidence confirms the result. Reconciliation after restart MUST derive membership from authoritative protocol state, not cached administrative state.

## 7. Signer and recovery interlocks

Lifecycle authorization MUST be isolated from signing authority. The review, registry and governance components MUST NOT possess operator private keys, invoke signer interfaces or change signer policy.

Before a validator is permitted to sign, the production integration MUST independently establish:
- committed active membership for the exact validator identity and chain/protocol;
- trusted consensus context and exact Malachite-native signing bytes;
- durable reservation commit before any key invocation;
- interception of every production signing path, including vote, proposal, vote extension and validator proof;
- anti-equivocation, anti-rollback and recovery checks.

Admission approval, activation-pending state, stale membership cache or governance exception MUST NOT satisfy these conditions. A signer MUST fail closed when membership or recovery evidence is missing, contradictory, stale or unverifiable.

The APEX Phase 20G2A preimage domain remains distinct from Malachite v0.8.2 native signing bytes. This phase does not alter ADR-0067/0068 or claim production signer integration exists.

## 8. Audit and privacy obligations

A conforming implementation MUST retain an append-only, tamper-evident relationship among:
- original submission and every evidence revision;
- reviewer assignments, conflicts and recusal events;
- findings, remediation and reassessment;
- eligibility results and baseline versions;
- governance decisions, approvals, denials, appeals and revocations;
- transition proposals, retries, rejection reasons and committed protocol proofs.

Corrections MUST be appended as new records; they MUST NOT erase prior decisions. Public disclosure and restricted evidence access must follow a ratified policy. Audit records MUST exclude secrets and private signing material. Retention, cryptographic construction, storage and disclosure rules remain open.

## 9. Required conformance tests

Before production implementation, tests MUST demonstrate:

1. state-domain separation: package completion, eligibility and approval never imply active membership;
2. identity/scope binding: cross-chain, cross-protocol, cross-operator and cross-validator replay is rejected;
3. stale-version rejection and deterministic conflict handling;
4. duplicate request idempotency and altered-envelope rejection;
5. reviewer conflict, unauthorized decision and invalid quorum/procedure rejection;
6. evidence revision lineage and prevention of evidence substitution/deletion;
7. no consensus membership mutation from administrative API/database actions;
8. pending status persists until valid committed transition proof is verified;
9. restart/replay reconstructs the same lifecycle and membership result;
10. failed, delayed and reordered transitions do not create duplicate or contradictory membership;
11. signer remains disabled before committed active membership and trusted context;
12. governance/review components cannot access keys or invoke signing;
13. signer reservation is durable before key use across all production signing paths;
14. rollback, journal conflict and incomplete recovery prevent signing;
15. suspension, exit and replacement cannot rewind or reuse signer reservations.

Tests against mocks or isolated harnesses are supplemental. They do not prove production engine integration, signer custody or live consensus correctness.

## 10. Unresolved decisions and non-goals

This phase selects no:
- constitutional authority, admission council, reviewer body or appeal authority;
- quorum, voting threshold, reviewer count, deadlines or emergency power;
- eligibility, stake, uptime, geography or provider-diversity threshold;
- activation delay, height/epoch boundary, exit queue or voting-power update rule;
- registry, database, API, schema, event format, proof format or retention period;
- signer backend, key format, custody arrangement or runtime.

It does not implement contracts, registries, state machines, APIs, migrations, consensus logic, signer code or runtime behavior.

## 11. Acceptance criteria

- Every lifecycle transition has explicit guard categories, durable audit outcome and fail-closed behavior.
- Administrative, governance, eligibility and consensus domains cannot be conflated.
- Concurrency, replay, idempotency and proof-of-effect requirements are explicit.
- Signer and recovery interlocks remain mandatory and independent of governance approval.
- Unratified decisions and unselected implementation details remain visibly unresolved.
- Documentation review and CI do not claim runtime correctness or close production gates.

**Current status:** proposed conformance contract only. Governance decisions OPEN. G2/G4/G5 BLOCKED. Production NO-GO.
