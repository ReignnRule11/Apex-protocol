# ADR-0070: Hybrid validator admission with objective eligibility and governance oversight

- **Status:** Proposed
- **Date:** 2026-11-08
- **Phase:** 20G2D14
- **Related:** ADR-0067, ADR-0068, ADR-0069

## Context

ADR-0069 records a hybrid validator operating model: protocol/ecosystem-operated validators may coexist with independent third-party operators, while each operator retains custody and signing authority for its own validator identities.

That operating model requires an admission framework. Purely discretionary admission risks opaque favoritism and capture; purely automatic admission without an accountable policy process leaves disputes, exceptions, sanctions and systemic concentration unaddressed. The existing governance constraints document leaves voting thresholds, quorum, timelock and veto undecided.

## Proposed decision

Adopt a **hybrid validator admission model** as the target policy:

1. Protocol-defined, published and objectively testable baseline requirements determine technical eligibility.
2. Routine qualification should be automated or independently reproducible wherever practical.
3. Governance oversight is reserved for policy changes, explicitly permitted exceptions, contested findings, appeals, sanctions and narrowly bounded emergency safety actions.
4. Protocol-operated and independent validators are subject to the same baseline requirements and protocol validity rules.
5. Admission authority does not confer custody of keys, signing authority, access to signer state, or power to bypass deterministic validator-set transitions.
6. All admission, denial, exception, suspension and removal decisions require an auditable rationale and evidence record.

This ADR is a proposal for governance review. It does not create or select a governance body, council, voting threshold, quorum, timelock, veto, emergency actor or admission contract.

## Authority constraints

- No private or undisclosed discretionary veto over an operator that satisfies the published baseline.
- No hidden admission or consensus-weight privilege for protocol-operated validators.
- No cross-operator private-key custody or signer-store access.
- No governance direction to sign conflicting or unsafe consensus messages.
- No arbitrary asset seizure or retroactive rewrite of finalized history.
- Safety-critical cryptographic, anti-equivocation and anti-rollback requirements cannot be waived through ordinary admission exceptions.
- Emergency suspension must be narrow, time-limited, evidenced, auditable and subject to mandatory review under a separately ratified procedure.
- Suspension and removal take effect only through protocol-defined validator-set transition semantics.

## Consequences

- A public eligibility checklist and reproducible conformance evidence are required.
- Operator identity, common control, infrastructure dependencies and fault-domain concentration need privacy-aware disclosure.
- Appeals, remediation, sanctions and emergency procedures need explicit due process.
- Validator-set activation, suspension, exit and replacement semantics must be specified before implementation.
- Numeric thresholds, governance authority and bootstrap transition policy remain open.
- G2/G4 remain BLOCKED; production remains NO-GO.

## Acceptance evidence

Change this ADR to **Accepted** only after:
- governance ratifies the authority model and decision procedures;
- objective admission requirements and evidence standards are specified;
- appeal, exception, sanction and emergency boundaries are approved;
- protocol-operated and independent operators are confirmed to face equivalent baseline rules;
- validator-set transition and activation semantics are deterministic and tested;
- privacy, common-control and infrastructure-diversity disclosure is reviewed;
- security review confirms admission governance cannot obtain independent signing authority or bypass signer safety controls.
