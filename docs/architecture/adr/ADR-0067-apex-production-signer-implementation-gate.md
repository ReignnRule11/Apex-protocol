# ADR-0067: APEX production signer implementation gate

- **Status:** Proposed
- **Date:** 2026-11-08
- **Related:** ADR-0042, ADR-0046, ADR-0063, ADR-0064, ADR-0065, ADR-0066
- **Phase:** 20G2D11

## Context

Phases 20G2D7–20G2D10 established the durable reservation storage contract, checkpoint/anti-rollback requirements, Malachite's native signer interception boundaries, and the upstream EngineBuilder injection seam. Those findings did not establish that APEX has a concrete production signer to wrap.

The root workspace contains:
- `apex-signer-safety`, which models typed signing coordinates and exact opaque message reservations;
- `apex-signer-redb`, which durably commits those reservations;
- `apex-consensus::consensus_message`, which constructs an experimental APEX preimage but does not sign;
- `apex-consensus::certificate`, which verifies Ed25519 signatures using public verification keys;
- a placeholder `apex-node` entry point with no networking or consensus activation.

The Malachite v0.8.2 dependencies and adapter experiments live in a separate experimental Cargo workspace. The inspected root-workspace Rust sources contain no APEX-owned Malachite `Signer<Ctx>` implementation, production key backend, or path connecting reservation commits to key invocation.

## Decision

Treat the absence of a production signer and its runtime construction path as a hard implementation gate.

Do not implement a speculative signer or wrapper before APEX selects:
1. the production validator process and signer ownership boundary;
2. the key custody/backend and its security/availability model;
3. the ratified signature scheme, key representation and validator identity profile;
4. the exact native Malachite byte construction and trusted context propagation;
5. the reservation-before-key-call integration and failure semantics;
6. complete coverage of direct key use, replay, validator proofs and vote extensions;
7. independent anti-rollback and recovery operations.

A future implementation must reserve the exact bytes consumed by the selected native key backend before invoking it. The APEX Phase 20G2A preimage must not be substituted for Malachite-native bytes. The reservation journal, preimage builder, certificate verifier, and experimental adapter are not production signers.

## Consequences

- The existing reservation crates remain prototypes with no production enforcement claim.
- No HSM/KMS/software-key/remote-signer choice is implied by the current dependency graph.
- No signer is enabled by this ADR, and no validator startup path is authorized.
- G2/G4 remain BLOCKED; release remains NO-GO.
- Promotion to Accepted requires a concrete custody decision, source-level call-path evidence, threat model, implementation review and adversarial tests.

## Rejected alternatives

- **Treat `ed25519-dalek` dependency presence as a signer:** rejected; the inspected consensus certificate code uses public-key verification.
- **Promote the Malachite test signer:** rejected; upstream test support is not an APEX production custody or integration decision.
- **Wrap only `ConsensusContext` injection:** rejected as proof of exact native-byte reservation and complete key-use interception.
- **Call the reservation store directly from an invented signer:** rejected; it would not establish actual application ownership, trusted context or complete backend coverage.
- **Enable the placeholder node:** rejected; it has no network or consensus runtime.

## Review gate

Before changing this ADR to Accepted, provide:
- selected production signer process and custody/backend architecture;
- complete inventory of signer constructors, all native signing operations and direct key-use paths;
- exact-byte trace for vote, proposal, vote extension and validator proof;
- trusted, non-forgeable APEX context binding;
- proof that reservation commit precedes every backend call and that failure blocks key use;
- replay, crash, concurrency, rotation, rollback, recovery and ambiguous-outcome tests;
- an independently protected monotonic anchor and operator recovery procedure.
