# ADR-0065: Malachite signer interception boundary

- **Status:** Proposed
- **Date:** 2026-11-08
- **Related:** ADR-0046, ADR-0055, ADR-0056, ADR-0063, ADR-0064
- **Phase:** 20G2D9

## Context

ADR-0064 requires durable reservation of the exact native signing bytes before any key backend is invoked, with complete coverage of every native signing entry point. The pinned Malachite v0.8.2 `Signer<Ctx>` API accepts typed votes, proposals, vote extensions and validator proofs. The concrete signing implementation is responsible for deriving the canonical bytes. The pinned Ed25519 key implementation exposes a lower-level byte-oriented signing operation. The consensus handler also has WAL replay helpers that may reuse a recorded signed vote or proposal without making a fresh key call.

## Decision

Treat the high-level signer trait and lower cryptographic key operation as separate interception boundaries.

A high-level decorator can observe typed intent but cannot claim exact-byte reservation without proving byte construction equivalence. A lower-level key wrapper can observe exact bytes but requires a trusted context channel that binds those bytes to chain, protocol, signer, signing domain and consensus coordinate. The implementation must prove that every key-use path is routed through the selected boundary.

Track ordinary votes, proposals, vote extensions and validator proofs as distinct signing domains. Vote-extension scope includes height, round, value ID and validator address. Validator proofs require an explicit separate scope or a documented exclusion from consensus anti-double-sign checkpoint coverage. Replay reuse must be represented separately from fresh key invocation.

The APEX Phase 20G2A preimage and Malachite native signing bytes remain distinct. No conversion or substitution is authorized.

## Consequences

- A trait-only wrapper is not sufficient evidence of exact-byte reservation.
- A byte-only key wrapper is not sufficient evidence of APEX identity/coordinate binding unless trusted context is carried into it.
- Coverage is not complete until all concrete signer implementations, construction paths and direct key-use paths are inventoried.
- Reservation must commit before the backend is called; reservation failure must prevent key invocation.
- Replay, reservation, key invocation, key result and signature emission are distinct observable states.
- The audit selects no codec, cryptographic suite, key custody, anchor technology, production wrapper or startup integration.
- G2/G4 remain BLOCKED; production remains NO-GO.

## Rejected alternatives

- **Wrap only `Signer<Ctx>`:** rejected as an exact-byte guarantee because its methods accept typed objects and delegate byte construction to implementations.
- **Wrap only the key backend:** rejected as a complete recovery contract without trusted consensus context and proof that all key-use paths pass through it.
- **Treat WAL replay as complete signer evidence:** rejected because replay reuse is conditional and the signer is invoked before WAL append on a miss.
- **Reuse APEX Phase 20G2A bytes as Malachite signing bytes:** rejected because the signing domains are distinct.

## Implementation gate

Before implementation, inventory concrete signer implementations and injection paths at the pinned revision; trace each method to its byte encoder and key backend; define a trusted context-carrying wrapper interface; specify domain-specific coordinate semantics; and test bypass, replay, concurrency, failure and crash boundaries. Do not promote this ADR to Accepted until the evidence is reviewed.
