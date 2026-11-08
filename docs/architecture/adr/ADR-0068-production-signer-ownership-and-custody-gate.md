# ADR-0068: Production signer ownership and custody selection gate

- **Status:** Proposed
- **Date:** 2026-11-08
- **Related:** ADR-0066, ADR-0067
- **Phase:** 20G2D12

## Context

The APEX source inventory establishes reservation policy and durable reservation storage, but no production signer, key backend, live validator runtime, or connection from reservation commit to cryptographic key invocation.

The pinned Malachite engine accepts an application-supplied signer through its consensus context, while validator-proof signing occurs before engine startup. The engine injection seam alone does not prove that all key use is intercepted or that exact native bytes are reserved. No current repository evidence selects a production validator process, custody model, signature profile, or independent anti-rollback mechanism.

## Decision

Treat production signer ownership and key custody as explicit architecture decisions that must be reviewed before implementation.

The protocol/security owners must select:
1. the validator process and complete signer request path;
2. the custody/backend model and its threat, availability, recovery and administrative boundaries;
3. the cryptographic suite, key representation and validator identity profile;
4. exact native byte construction and trusted context for votes, proposals, vote extensions and validator proofs;
5. a complete key-use interception strategy, including paths outside the engine injection seam;
6. durable reservation-before-key-call behavior and ambiguous-failure semantics;
7. an independently protected monotonic anti-rollback anchor and recovery process;
8. operational ownership for enrollment, rotation, revocation, incident response and disaster recovery.

This ADR does not choose in-process software keys, HSM, KMS, remote signing, threshold/MPC, or any other backend. It does not authorize key generation, key import, signing, validator startup, or release.

## Consequences

- No production signer architecture is implied by current dependencies or experimental code.
- The existing reservation crates remain unconnected prototypes until a reviewed implementation proves enforcement.
- Test signers remain test-only.
- The APEX Phase 20G2A preimage cannot be substituted for Malachite-native signing bytes.
- Vote-extension and validator-proof operations require explicit coverage; replayed signatures are not fresh key calls.
- G2/G4 remain BLOCKED and release remains NO-GO.

## Acceptance evidence

Change this ADR to **Accepted** only when a concrete architecture decision provides:
- selected runtime ownership and backend/custody;
- source-level complete call graph and exact byte trace for every signing operation;
- trusted context and authorization contract;
- reservation commit ordering and fail-closed behavior proof;
- crash, concurrency, replay, timeout, rotation, rollback and recovery tests;
- independently protected anti-rollback anchor and operational recovery runbook;
- security review and named operational ownership.

A proposed implementation or successful unit test without the above evidence is insufficient.
