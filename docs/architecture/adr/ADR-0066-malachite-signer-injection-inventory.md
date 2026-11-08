# ADR-0066: Concrete Malachite signer injection and coverage inventory

- **Status:** Proposed
- **Date:** 2026-11-08
- **Related:** ADR-0063, ADR-0064, ADR-0065
- **Phase:** 20G2D10

## Context

ADR-0065 separated the typed `Signer<Ctx>` interface from the lower byte-oriented key operation. Phase 20G2D10 inspected the pinned Malachite v0.8.2 application-channel builder, convenience start function, consensus actor, signing trait, test signer and replay helpers.

The default application integration supplies `Box<dyn Signer<Ctx>>` through `ConsensusContext::new_validator`; `start_engine` forwards that context through `EngineBuilder`. The consensus actor owns the optional signer and dispatches vote, proposal and vote-extension signing effects to it. Validator-proof signing is performed by the application before it constructs the validator network identity. The engine can also reuse recorded vote/proposal signatures during WAL replay without a fresh signer call.

The pinned upstream test crate contains an Ed25519 test signer. The APEX repository does not yet select a production signer, key backend or custody model.

## Decision

Record the application-supplied signer in `ConsensusContext` as the default high-level integration seam, not as proof of complete interception. Treat the pre-engine validator-proof call and all application-owned direct key operations as separate coverage paths. Treat replayed signature reuse separately from fresh key invocation.

Do not implement a wrapper until the concrete APEX application signer, key backend, custody model, signing schemes and all construction/use paths are selected and audited.

## Consequences

- APEX must install any future high-level decorator at the application signer construction/injection point.
- A decorator at that point still needs proof of canonical byte equivalence or a trusted handoff to the exact-byte backend.
- The upstream test signer is not a production signer and does not close APEX's implementation inventory.
- Validator proofs require a distinct pre-engine signing path and explicit recovery scope.
- Replay hits do not invoke the configured signer for that event and cannot be treated as evidence of historical reservation completeness.
- No production wrapper, key custody, codec, checkpoint, schema or startup behavior is selected.
- G2/G4 remain BLOCKED; production remains NO-GO.

## Implementation gate

Inventory the downstream application's signer implementations and all direct key-use paths; trace every signing operation to its actual backend bytes; design trusted context propagation; then specify and test durable pre-invocation reservation, replay, bypass, crash, concurrency and recovery behavior. Do not mark this ADR Accepted until those items are reviewed.
