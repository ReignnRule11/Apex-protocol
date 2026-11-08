# Phase 20G2D13 — hybrid validator ownership architecture

**Status:** Architecture direction recorded for review; implementation and custody decisions remain gated.  
**Base main:** `5ee7ef1251f7f211d678f008b8ebb0f2c760f81c`.  
**Related:** Phase 20G2D12; ADR-0068.  
**Release impact:** G2/G4 remain BLOCKED; production remains NO-GO.

## Direction

APEX will target a **hybrid validator operating model**:

1. **Protocol/ecosystem-operated validators** may be operated by the APEX founding/operating organization for bootstrap, controlled infrastructure, and operational continuity.
2. **Independent third-party validators** may be operated by separately governed organizations or individuals that meet published admission, security, availability, and protocol requirements.

This is an operating-topology decision, not a decision to centralize signing. The two cohorts participate in the same consensus protocol and are subject to the same protocol-level validity rules. The APEX organization does not receive custody of independent operators' private keys merely because it coordinates the protocol or operates some validators.

## Trust and authority boundaries

| Concern | Protocol/ecosystem operator | Independent validator operator |
|---|---|---|
| Validator process | Operates and secures its own process | Operates and secures its own process |
| Validator key custody | Controls keys for its own validator identities | Controls keys for its own validator identities |
| Signing authorization | Enforced within its own approved signer boundary | Enforced within its own approved signer boundary |
| Host, network and backups | Responsible for its own infrastructure | Responsible for its own infrastructure |
| Protocol upgrades | Follows published governance/release process | Follows same published governance/release process |
| Monitoring and incident response | Reports and responds for its own validators | Reports and responds for its own validators |
| Other operators' private keys | No default access or authority | No default access or authority |

The protocol organization may define validator admission, minimum operational requirements, software releases, slashing/accountability rules if adopted by protocol governance, and incident disclosure procedures. Those powers do not imply unilateral access to validator signing keys or authority to sign as another operator.

## Non-negotiable architecture constraints

- Each validator identity has an explicit accountable operator and independently controlled signing authority.
- No shared private key, shared hot-wallet identity, or centrally held fleet key across independent operators.
- No central signing proxy is assumed. A remote signer may be evaluated as an operator-selected backend only if the operator retains explicit control and the threat model, authentication, authorization, availability and audit boundaries are approved.
- Protocol-operated validators and independent validators must not share signer state, reservation databases, key material, or administrative credentials by default.
- Validator identity, consensus power, admission, and operator attribution must be explicit and auditable; avoid hidden preferential authority for protocol-operated nodes.
- Consensus safety and liveness must not depend on the protocol organization being continuously available, except for any explicitly specified bootstrap/governance functions.
- Key rotation, revocation, operator exit, compromise response, and validator-set transitions must be protocol-defined and tested.
- Deployment diversity (operator, infrastructure provider, region, network, and administrative domain) must be measured rather than assumed from a count of validator processes.

## Signer architecture consequences

The hybrid model requires a **per-operator signer boundary**, not one universal custody product. The production signer design must support at least two deployment profiles without changing consensus signing semantics:

1. **Protocol-operated profile:** the organization selects and operates the signer runtime and custody model for its own validator identities.
2. **Independent-operator profile:** each validator operator selects an approved signer deployment and custody model for its own identity, subject to protocol interoperability and security requirements.

These profiles may use different backend technologies only if they implement the same ratified cryptographic profile, exact native-byte contract, trusted context binding, reservation-before-key-call semantics, and operational evidence requirements. The protocol must not require independent operators to disclose private key material to the protocol organization.

The current repository does not yet prove these contracts. This phase does not select software keys, HSM, KMS, remote signing, threshold/MPC, or a particular validator runtime.

## Required follow-on decisions

Before implementation or production enablement, separate decisions are still required for:

- validator admission, operator identity and accountability;
- validator-set bootstrap and transition governance;
- fault-domain and geographic diversity objectives;
- protocol-operated validator custody/backend;
- independent-operator approved backend profiles and conformance requirements;
- cryptographic suite, key format, validator identity derivation and rotation;
- native Malachite byte traces for votes, proposals, vote extensions and validator proofs;
- trusted context propagation and complete interception of every key-use path;
- durable reservation ordering, ambiguous backend outcomes and replay handling;
- independently protected anti-rollback and recovery;
- monitoring, incident response, suspension, exit and replacement of operators.

## Explicitly not decided

- Number or percentage of protocol-operated versus independent validators.
- Minimum independent-operator count or decentralization threshold.
- Geographic/provider distribution targets.
- Whether protocol-operated validators are temporary bootstrap validators or permanent participants.
- Specific key custody products, signer runtime, HSM/KMS/remote signer, or cryptographic suite.
- Validator admission authority, governance voting weights, or emergency powers.

These require threat modelling, economic/security analysis, governance ratification and measurable acceptance criteria. Do not invent numeric thresholds in implementation.

## Exit criteria

- Hybrid operating topology is documented without implying centralized custody.
- The two operator profiles and trust boundaries are explicit.
- Per-operator key custody and no-cross-operator-key-access constraints are explicit.
- Outstanding governance, diversity, custody, cryptographic and signer-enforcement decisions remain visible.
- No key, signer, runtime, schema, WAL or startup behavior changes.
- G2/G4 remain BLOCKED; production remains NO-GO.

## Next permitted work

Develop a validator-operator governance and admission specification, then a per-operator signer conformance contract. Only after those are reviewed should implementation work select concrete signer backends and runtime integration.
