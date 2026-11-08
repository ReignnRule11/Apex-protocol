# Phase 20G2D14 — validator governance and admission specification

**Status:** Proposed policy and decision gate; not governance-ratified or implemented.  
**Base main:** `8ad8e4ef95eff7386b473790982d14344120774e`.  
**Related:** Phase 20G2D13; ADR-0069; Governance Constraints.  
**Release impact:** G2/G4 remain BLOCKED; production remains NO-GO.

## Objective

Translate the hybrid validator operating model into a transparent admission and accountability framework without granting the protocol organization unilateral custody or signing authority over independent operators.

This phase records a **hybrid admission model as the recommended proposal**:

- Protocol-defined, objectively testable baseline requirements determine technical eligibility.
- Governance oversight handles policy exceptions, disputes, sanctions and changes to the admission framework under publicly specified procedures.
- Governance must not silently waive safety-critical requirements or privately privilege protocol-operated validators.

This is a proposed design direction, not a claim that APEX governance has ratified an authority, council, voting system or emergency power.

## 1. Admission principles

1. **Equal protocol rules:** protocol-operated and independent validators satisfy the same consensus validity, cryptographic and safety requirements.
2. **Objective baseline:** requirements are published, testable, versioned and applied consistently.
3. **Operator accountability:** every validator identity maps to a responsible operator and declared operational contact, subject to appropriate privacy protections.
4. **Least authority:** admission authority does not imply custody of keys, control of infrastructure, or power to produce signatures for an operator.
5. **Transparent decisions:** admission, denial, suspension and removal decisions carry a public reason code and evidence summary, except narrowly redacted sensitive details.
6. **Due process:** operators receive notice, an opportunity to respond, and an appeal path except where a narrowly defined immediate safety action is necessary.
7. **No hidden privilege:** organizational affiliation with APEX does not grant extra consensus weight, relaxed eligibility, privileged signing, or immunity from published rules.
8. **Measured decentralization:** operator count is not treated as fault-domain diversity; ownership, administrative control, hosting, region, network and dependency concentration must be assessed.

## 2. Proposed admission lifecycle

| Stage | Required evidence | Result |
|---|---|---|
| Application | Operator identity/accountability declaration; validator identity; requested role | Application recorded |
| Eligibility validation | Published software, protocol, security and operational checks | Eligible, incomplete, or ineligible |
| Operational verification | Reachability, monitoring, incident contact, recovery and upgrade readiness evidence | Verification result |
| Admission decision | Baseline result and any permitted governance review | Approved or reasoned denial |
| Activation | Valid protocol-defined validator-set transition | Validator becomes active only at the specified effective point |
| Ongoing compliance | Periodic and event-triggered evidence | Continue, remediate, suspend or remove |
| Exit/replacement | Key retirement/revocation and validator-set transition evidence | Identity safely deactivated/replaced |

No application or approval alone may activate consensus power. Activation must occur through a deterministic, protocol-defined validator-set transition whose effective height/epoch and authorization rules are specified separately.

## 3. Baseline eligibility categories

The following categories are required, but numerical thresholds and exact evidence formats remain open until threat modelling and protocol parameters are approved.

### Operator identity and accountability
- A unique operator record and accountable legal/personhood or organizational representation appropriate to the eventual jurisdiction and privacy model.
- A contact channel for security incidents and protocol notices.
- Disclosure of common control across nominally separate operators where relevant to fault-domain analysis.
- A documented process for changing operator ownership or control.

### Protocol and software conformance
- Run an approved, reproducibly identified release and configuration.
- Pass versioned consensus, signer and validator conformance suites.
- Reject unsupported protocol versions and unsafe configuration.
- Demonstrate controlled upgrade and rollback procedures; rollback must not permit signer-state rollback.

### Security and key custody
- Operator-controlled signing authority for its validator identities.
- No disclosure of private key material to the protocol organization as an admission condition.
- Per-operator signer isolation, access control, backup/recovery and key lifecycle evidence.
- Exact native signing bytes and trusted context conformance for every signing path.
- Durable reservation-before-key-call, replay protection, anti-rollback and fail-closed ambiguous-outcome behavior.
- Incident reporting and compromise containment procedures.

These are requirements to be proven; current repository audits do not establish that a production signer meets them.

### Operational reliability
- Published availability and response objectives once approved.
- Monitoring, alerting, time synchronization and capacity evidence.
- Tested backup, restore and disaster recovery.
- Network and infrastructure dependency disclosure sufficient to assess concentration.
- Security patch and protocol upgrade service-level expectations.

### Governance and conduct
- Acceptance of published protocol rules and evidence obligations.
- Conflict-of-interest and common-control disclosures relevant to validator independence.
- Cooperation with narrowly scoped security investigations.
- No requirement to endorse a political position, transfer private keys, or grant unrelated administrative access.

## 4. Governance oversight boundary

The proposed hybrid model separates routine technical qualification from policy oversight.

**Protocol-defined eligibility checks** should be automated or independently reproducible wherever possible. A successful baseline result should not be subject to an undisclosed discretionary veto.

**Governance oversight** may be required for:
- changing baseline admission or security requirements;
- approving a documented exception where the framework explicitly permits one;
- resolving contested eligibility findings and appeals;
- imposing or lifting protocol-level sanctions;
- authorizing emergency suspension under a narrowly defined safety rule;
- reviewing concentration, conflicts of interest and systemic risk.

Governance may not:
- access, export or operate an independent operator's private keys;
- direct a validator to sign a conflicting or unsafe message;
- privately alter consensus weight or validity rules for a named operator;
- bypass deterministic validator-set transition rules;
- impose an unbounded or unaudited emergency suspension;
- retroactively rewrite finalized consensus history.

Any exception must identify the requirement, evidence, rationale, scope, expiry, compensating controls, approving authority and appeal/review path. Safety-critical cryptographic and anti-equivocation requirements are not waivable through an ordinary admission exception.

## 5. Denial, remediation, suspension and removal

- **Incomplete application:** return a specific deficiency list; do not treat silence as approval.
- **Baseline failure:** provide reproducible evidence and a remediation path where the failure is remediable.
- **Ordinary non-compliance:** notice, remediation window, reassessment and reasoned decision.
- **Immediate safety risk:** a narrowly scoped temporary action may be taken only under a future ratified emergency procedure; record evidence, authority, start time, expiry and mandatory review.
- **Suspension/removal:** never rely on an informal operator instruction. Use a protocol-defined validator-set transition with explicit effective point and audit trail.
- **Appeal:** an independent review path must exist; the decision-maker and timelines remain to be ratified.
- **Key compromise:** disable or rotate the affected validator identity through a safe transition; do not transfer its key to governance or reuse its signing coordinates.

Suspension of an operator's participation must not imply a right to seize assets or arbitrarily alter balances. Any economic penalties or slashing require a separately specified and ratified protocol rule, evidence standard and appeal mechanism.

## 6. Protocol-operated validators

Protocol/ecosystem-operated validators are subject to the same baseline eligibility, security, reporting and transition requirements as independent operators.

Their organizational owner must be disclosed. They must not receive:
- automatic admission outside the published process;
- additional consensus weight because of organizational ownership;
- privileged access to independent operators' keys or signer stores;
- immunity from suspension, remediation or removal;
- unilateral authority to change admission rules.

If protocol-operated validators are used for bootstrap, the bootstrap trust assumptions, duration/exit conditions and transition to a sufficiently independent validator set must be explicit before public testnet or production authorization.

## 7. Diversity and concentration evidence

A future admission registry should track, with appropriate privacy and security controls:
- beneficial/common operator control;
- administrative and signing-control domains;
- hosting provider and cloud account dependencies;
- geographic and jurisdictional concentration;
- network, DNS, RPC and upstream dependency concentration;
- client/software version and release concentration;
- correlated backup, monitoring and identity-provider dependencies.

No numerical diversity threshold is selected here. A validator count alone is not evidence of decentralization. Thresholds must follow a documented threat model, economic analysis and governance ratification.

## 8. Required protocol contracts before implementation

The following remain separate engineering and governance gates:

1. Canonical operator and validator identity model.
2. Admission registry schema, provenance and privacy boundaries.
3. Validator-set update authorization, ordering and activation semantics.
4. Deterministic effective height/epoch and transition failure handling.
5. Suspension, revocation, rotation, exit and replacement semantics.
6. Appeal and emergency authority model, including limits and auditability.
7. Per-operator signer conformance and key lifecycle.
8. Exact Malachite native signing-byte and trusted-context evidence.
9. Reservation-before-key-call, replay, anti-rollback and recovery proofs.
10. Fault-domain metrics and measurable diversity objectives.

Do not implement an admission contract, governance authority, registry schema, slashing rule or signer backend based solely on this proposal.

## 9. Open decisions

- Governance body or mechanism with authority to ratify admission policy.
- Whether routine approval is automatic on passing checks or requires a bounded review window.
- Identity verification and privacy model.
- Exact baseline thresholds and evidence retention periods.
- Appeal reviewer independence and timelines.
- Emergency suspension trigger, authorized actors, expiry and review.
- Whether and how economic penalties are used.
- Bootstrap validator sunset/transition conditions.
- Diversity thresholds and disclosure granularity.

The existing governance constraints document explicitly leaves voting thresholds, quorum, timelock and veto undecided. This phase does not invent those parameters.

## Exit criteria

- Proposed hybrid admission model and authority boundaries documented.
- Baseline eligibility categories and lifecycle stages specified.
- Due process, emergency constraints and audit requirements documented.
- Protocol-operated validators explicitly subject to the same baseline.
- Open governance and quantitative decisions remain visible.
- No code, runtime, key, schema, validator-set, consensus or signer behavior changed.
- G2/G4 remain BLOCKED; production remains NO-GO.

## Next permitted work

Obtain governance review of the proposed authority model and publish the per-operator validator admission evidence/conformance contract. Implementation remains gated on ratified governance, deterministic validator-set transition semantics, and production signer/recovery proof.
