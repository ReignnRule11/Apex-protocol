# ADR-0069: Hybrid validator operating model and per-operator signing authority

- **Status:** Proposed
- **Date:** 2026-11-08
- **Related:** ADR-0067, ADR-0068
- **Phase:** 20G2D13

## Context

APEX needs an explicit validator operating model before it can select production signer ownership and custody. A single centrally operated validator fleet would constrain independent participation; an exclusively third-party model would leave bootstrap and protocol-operated infrastructure questions unanswered.

The signer absence and architecture decision reviews established that no production signer, custody backend, complete key-use path, or anti-rollback implementation is currently proven.

## Decision

Adopt a **hybrid validator operating model as the target architecture**:

1. The APEX protocol/ecosystem organization may operate validators for bootstrap and its own operational purposes.
2. Independent third-party operators may operate validators under published, consistently applied protocol admission and security requirements.
3. Each operator is accountable for, and independently controls, the signing authority and key custody for its own validator identities.
4. Protocol coordination, software releases, validator admission, and governance do not confer custody of independent operators' private keys.
5. Protocol-operated and independent validators use the same consensus validity rules and ratified cryptographic/native-byte contracts.

This ADR selects an operating topology only. It does not select a signer runtime, custody technology, cryptographic suite, operator ratio, admission authority, or governance power model.

## Security constraints

- No shared private key or fleet-wide signing identity across independent operators.
- No cross-operator access to private keys, signer credentials, reservation stores, or administrative credentials by default.
- A remote signer is not the default architecture and cannot be treated as independent custody merely because it runs in another process or service.
- Each operator's signer must enforce trusted context, exact native bytes, durable reservation-before-key-call, replay handling, and fail-closed ambiguous outcomes.
- Any backend diversity must preserve identical protocol-level signing semantics and pass conformance tests.
- Operational and infrastructure diversity must be measured; multiple validators under one administrative or cloud account do not establish independent fault domains.

## Consequences

- The signer architecture must define at least a protocol-operated deployment profile and an independent-operator deployment profile.
- APEX must publish operator admission, accountability, key lifecycle, upgrade, monitoring, incident response, suspension, exit and replacement requirements.
- The protocol organization must not require independent operators to disclose private key material.
- Protocol-operated validators must not receive hidden consensus privileges merely by virtue of organizational ownership.
- Numeric validator ratios, minimum independent-operator counts, and geographic/provider thresholds remain open pending threat and economic analysis.
- G2/G4 remain BLOCKED; production remains NO-GO.

## Acceptance evidence

Change this ADR to **Accepted** only after:
- governance ratifies the operator model and admission authority;
- threat modelling defines trust boundaries and collusion/failure assumptions;
- diversity objectives and measurable fault-domain criteria are approved;
- per-operator signer conformance and key lifecycle requirements are specified;
- exact native signing bytes and trusted context are proven for every signing operation;
- reservation ordering, anti-rollback, recovery and incident procedures are tested;
- security review confirms no cross-operator key custody or hidden privileged signing path.
