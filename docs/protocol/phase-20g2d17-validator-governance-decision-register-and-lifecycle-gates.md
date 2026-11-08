# Phase 20G2D17 — validator governance decision register and deterministic lifecycle gates

**Status:** Proposed decision register and lifecycle contract; not governance-ratified or implemented.  
**Parent:** Phase 20G2D16; ADR-0070, ADR-0071 and ADR-0072.  
**Branch base:** `ced39800b674c8d27251820b912a4e82d39f0baf`.  
**Release impact:** G2/G4 remain BLOCKED; production remains NO-GO.

## 1. Purpose

Convert the unresolved governance and lifecycle questions from the hybrid admission model into a controlled decision register and a protocol-neutral lifecycle contract. The goal is to prevent implementation from silently choosing authority, deadlines, voting rules, activation timing or emergency powers.

This phase defines what must be decided, what evidence a decision requires, which invariants cannot be waived, and what a future implementation must prove. It does not make those governance decisions on behalf of the protocol.

## 2. Normative language and status

The terms MUST, MUST NOT, SHOULD and MAY describe requirements for any future ratified implementation. They do not imply that an unratified authority currently exists.

Decision status:
- **OPEN:** no approved decision exists.
- **PROPOSED:** an option is documented but not approved.
- **RATIFIED:** approved by the authority and process established by the governing constitution/policy.
- **IMPLEMENTED:** code and operational controls implement a ratified decision.
- **VERIFIED:** independent evidence demonstrates the implementation satisfies the decision.

A decision cannot skip directly from PROPOSED to IMPLEMENTED. A merged documentation PR does not constitute governance ratification.

## 3. Governance decision register

All rows below are **OPEN** unless explicitly updated by a later ratified governance record. No default, implied authority or numerical value is selected.

| ID | Decision required | Minimum decision record | Blocking consequence |
|---|---|---|---|
| GOV-01 | Constitutional source and authority to establish validator admission policy | Source of authority, scope, amendment process, precedence and public reference | No admission policy can be treated as binding |
| GOV-02 | Admission decision authority | Composition/selection method, mandate, term, removal, accountability and separation from operators | No admission approval can be treated as authorized |
| GOV-03 | Reviewer appointment and independence | Eligibility, assignment, conflict categories, recusal, replacement and audit | No independent evidence assessment can be relied upon |
| GOV-04 | Decision procedure | Required reviewers, quorum, voting/consensus rule, tie handling, abstention and record publication | No final governance decision |
| GOV-05 | Baseline eligibility criteria | Versioned objective requirements, evidence methods, pass/fail rules and change control | No technical eligibility determination |
| GOV-06 | Exception boundaries | Permitted grounds, authority, scope, compensating controls, expiry and prohibited waivers | No exceptions; mandatory safety requirements remain absolute |
| GOV-07 | Remediation and reassessment | Finding severity, response window, evidence refresh, reassessment scope and status during remediation | No deadline or interim status may be inferred |
| GOV-08 | Appeals and disputes | Grounds, standing, filing window, independent reviewer, review standard, remedy, finality and stay effect | No appeal outcome or stay effect may be inferred |
| GOV-09 | Public disclosure and confidentiality | Public decision fields, redaction criteria, restricted evidence access, audit logging and challenge process | No disclosure of restricted evidence beyond approved policy |
| GOV-10 | Suspension and emergency action | Objective triggers, authorized actor, necessity/proportionality, duration, review, notice and protocol effect | No emergency authority or immediate consensus effect |
| GOV-11 | Voluntary exit, removal and replacement | Request/decision authority, notice, pending-exit behavior, transition point, identity continuity and recovery obligations | No lifecycle transition can be operationally inferred |
| GOV-12 | Validator-set transition semantics | Deterministic activation/removal point, ordering, duplicate handling, epoch/height binding and failure behavior | No admission decision may activate or remove a validator |
| GOV-13 | Operator and fault-domain disclosure | Common-control definition, dependency taxonomy, privacy limits and concentration measurement | No numerical diversity claim or threshold |
| GOV-14 | Evidence and decision record integrity | Canonical identity, versioning, integrity/authentication, provenance, retention and independent audit | No production evidence registry or authoritative record |
| GOV-15 | Implementation and assurance ownership | Code ownership, security review independence, operational responsibility and incident accountability | No production workflow implementation |

### Decision record template

Every future ratified decision MUST record:
1. stable decision ID and version;
2. exact question and scope;
3. constitutional/policy authority and approval procedure;
4. options considered, rationale and rejected alternatives;
5. affected protocol invariants and threat-model analysis;
6. implementation and operational consequences;
7. migration/activation plan, rollback limits and compatibility;
8. public disclosure and confidentiality treatment;
9. review date, amendment path and superseded decision references;
10. evidence of approval and independent verification status.

A decision record MUST NOT contain private keys, seed phrases, reusable credentials or secrets.

## 4. Validator lifecycle model — abstract states

The following is a proposed state vocabulary, not an activated state machine and not a selection of governance authority or timing.

1. **UNREGISTERED** — no accepted operator/validator identity record exists.
2. **SUBMITTED** — an operator has submitted an identity and evidence package.
3. **UNDER_REVIEW** — intake and independent assessment are in progress.
4. **REMEDIATION_REQUIRED** — one or more findings require correction or additional evidence.
5. **ELIGIBLE** — the applicable versioned technical baseline has been independently shown to pass.
6. **ADMISSION_APPROVED** — a properly authorized governance decision approves the exact operator, validator identity and scope.
7. **ACTIVATION_PENDING** — all protocol-defined activation prerequisites are satisfied and the exact proposed validator-set transition is recorded.
8. **ACTIVE** — the validator is in the consensus validator set following the deterministic protocol transition.
9. **SUSPENSION_PENDING** — an authorized suspension decision exists but has not yet taken effect through protocol rules.
10. **EXIT_PENDING** — a voluntary exit or approved removal is awaiting its protocol-defined transition.
11. **INACTIVE** — the identity is not currently a consensus participant.
12. **REJECTED / WITHDRAWN** — the relevant application was rejected or withdrawn; this is an application disposition, not a consensus state.
13. **REVOKED** — an admission approval was withdrawn under an authorized process; any consensus effect remains a separate protocol transition.

The terms are deliberately separated: administrative disposition does not equal consensus membership. An operator may have an approved application while its validator is not active; an active validator may be subject to a pending governance process whose consensus effect has not yet occurred.

## 5. Transition contract

Any implementation MUST define a deterministic transition table before activation. At minimum:

| Transition | Required guard | Forbidden shortcut |
|---|---|---|
| UNREGISTERED → SUBMITTED | Stable identity, scoped package and submission provenance | Creating active consensus membership |
| SUBMITTED → UNDER_REVIEW | Intake validation and controlled reviewer assignment | Treating upload as approval |
| UNDER_REVIEW → REMEDIATION_REQUIRED | Recorded finding tied to requirement and evidence revision | Deleting or overwriting original evidence |
| UNDER_REVIEW → ELIGIBLE | All mandatory baseline criteria pass with scoped evidence | Reviewer discretion to waive a safety-critical failure |
| ELIGIBLE → ADMISSION_APPROVED | Ratified authority, valid procedure, exact identity/scope and immutable decision record | Automated approval from package completeness alone |
| ADMISSION_APPROVED → ACTIVATION_PENDING | All ratified protocol activation prerequisites and transition inputs are met | Governance API directly mutating validator set |
| ACTIVATION_PENDING → ACTIVE | Deterministic consensus transition commits at protocol-defined point | Off-chain timestamp, operator assertion or UI state |
| ACTIVE → SUSPENSION_PENDING | Authorized, evidenced and scoped suspension decision | Immediate deletion or arbitrary weight mutation |
| ACTIVE → EXIT_PENDING | Valid voluntary exit or authorized removal request | Reusing identity or signer state for a replacement |
| SUSPENSION_PENDING / EXIT_PENDING → INACTIVE | Applicable deterministic protocol transition takes effect | Treating decision issuance as transition completion |
| Any application state → WITHDRAWN | Authenticated withdrawal by authorized applicant | Rewriting prior review history |
| ADMISSION_APPROVED → REVOKED | Ratified revocation process and immutable reason record | Implicit revocation from an unrelated package edit |

Exact state transitions, height/epoch boundaries, validator power updates, ordering, activation delays, cancellation behavior and failure recovery remain OPEN. This table is a minimum design checklist, not a complete protocol algorithm.

## 6. Consensus and signer safety invariants

A future implementation MUST preserve all of the following:

- **No governance-to-consensus bypass:** only protocol-defined deterministic transitions alter validator membership or voting power.
- **No off-chain activation:** a database row, API response, dashboard label, signed admission letter or governance event is not proof of active membership.
- **Exact identity binding:** every decision and transition binds chain ID, protocol version, validator identity, operator identity and applicable scope using a canonical representation selected by a future ADR.
- **No identity substitution:** evidence or approval for one validator identity cannot be replayed for another identity or operator.
- **No cross-operator custody:** admission/review systems cannot obtain, export or invoke operator signing keys.
- **No safety waiver:** no governance exception can authorize conflicting signatures, bypass anti-equivocation, rewind signer reservations or defeat anti-rollback/recovery protections.
- **No premature signer enablement:** a validator cannot sign merely because it is submitted, under review, eligible, approved or activation-pending.
- **No false deactivation:** administrative suspension/revocation does not claim consensus removal until the deterministic transition is confirmed.
- **Audit continuity:** decisions, evidence revisions, transition proposals, committed transition identities and appeal outcomes remain linked without rewriting history.
- **Fail closed:** missing, conflicting, stale or unverifiable authorization/transition evidence blocks the affected transition.

The APEX signer reservation/preimage domain and Malachite v0.8.2 native signing-byte domain remain distinct. This phase does not alter or relax the production signer requirements recorded in ADR-0067/0068.

## 7. Required adversarial and conformance tests

Before implementation can be considered production-grade, tests MUST cover:

### Governance and decision integrity
- decision submitted by an unauthorized actor;
- invalid quorum, missing approval or conflicted reviewer;
- replay of a decision against a different chain, protocol, operator or validator identity;
- duplicate, contradictory or superseded decisions;
- decision mutation after publication;
- appeal reviewer overlap with original decision-makers;
- exception attempting to waive a prohibited safety invariant.

### Lifecycle and consensus boundary
- package completion cannot activate;
- technical eligibility cannot activate;
- admission approval cannot activate without protocol prerequisites;
- forged or stale activation proof is rejected;
- duplicate activation and duplicate removal requests are deterministic and idempotent or explicitly rejected;
- out-of-order, stale-height and conflicting transition requests fail closed;
- suspension decision is not reported as effective before consensus transition;
- voluntary exit and removal cannot cause double membership or identity reuse;
- replacement validator has independent identity and signer safety history;
- restart/replay reproduces the same lifecycle and validator-set result.

### Signer and recovery boundary
- admission/review services cannot access signer secrets or call signing APIs;
- no signer invocation before durable reservation commit;
- exact native signing bytes and every production signing path remain intercepted;
- rollback, stale recovery evidence and signer journal conflicts prevent signing;
- governance decisions cannot rewind, delete or reuse signer reservations.

Tests MUST run against the actual production integration path once it exists. Mocks and isolated harnesses are useful but cannot be represented as proof of production signer integration or live consensus transition correctness.

## 8. Evidence required to close future gates

| Gate | Required evidence |
|---|---|
| Governance ratification | Verifiable authority, adopted policy version, approval record and public decision procedure |
| Eligibility | Published versioned criteria, reproducible test methods and independent assessment |
| Review integrity | Conflict controls, assignment audit, evidence provenance and immutable decision history |
| Lifecycle correctness | Formal transition table, deterministic implementation, state-machine/property tests and replay evidence |
| Consensus correctness (G2) | Production engine integration, validator-set transition conformance, adversarial multi-process tests and independent review |
| Durable operation (G4) | Production signer, exact native bytes, durable reservation-before-key-call, anti-rollback, recovery and crash/restart evidence |
| Security assurance (G5) | Independent threat model, access-control review, adversarial testing and remediation evidence |

Passing documentation CI does not satisfy any runtime gate.

## 9. Explicit non-decisions

This phase does not choose:
- an admission council, DAO, foundation, multisig or other authority;
- membership, reviewer count, quorum, voting threshold, veto or timelock;
- numerical eligibility, stake, uptime, geographic or provider-diversity thresholds;
- appeal windows, remediation deadlines, suspension durations or emergency powers;
- validator activation delay, epoch/height semantics, power update rules or exit queue;
- registry, database, contract, API, event schema, cryptographic format or retention period;
- production signer, key backend, key format, custody model or consensus runtime implementation.

## 10. Acceptance criteria

This phase is a complete proposed decision/lifecycle specification only when:
- every open decision is explicit and has a required decision-record shape;
- state vocabulary distinguishes administrative, evidentiary and consensus states;
- transition guards and forbidden shortcuts are documented;
- signer and recovery invariants are preserved;
- adversarial test requirements are mapped to future implementation gates;
- no unratified authority, threshold or protocol behavior is presented as settled.

**Current status:** governance decisions OPEN; lifecycle contract PROPOSED; no implementation authorized by this document. G2/G4/G5 remain BLOCKED; production NO-GO.
