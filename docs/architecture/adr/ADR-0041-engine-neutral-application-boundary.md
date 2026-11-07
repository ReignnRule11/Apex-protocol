# ADR-0041: Engine-Neutral Consensus/Application Boundary

**Status:** Accepted for protocol-boundary prototyping; production finality NOT approved.
**Date:** 2026-11-07

## Context

APEX is Rust-first and is evaluating Malachite v0.8.2 in an isolated workspace. Protocol/application logic must not inherit an engine's wire format, identity model, signing assumptions or local finality semantics by accident.

## Decision

Add `apex-consensus::application_boundary` as an engine-neutral contract:

1. Bind chain ID and protocol version explicitly.
2. Check candidate height and parent linkage with checked arithmetic.
3. Invoke deterministic application validation before representing a proposal as validated.
4. Require a separate engine-specific `FinalityVerifier` to verify a claim for the exact validated block identity.
5. Construct the application-visible `VerifiedFinalizedBlock` only after verifier success.
6. Invoke application commit only after the above steps.

Proposal and proof bytes are bounded by caller-supplied limits. These limits are not ratified protocol constants.

## Trust and failure model

The verifier is a security-critical adapter contract, not an implementation. It must authenticate chain/version/height/hash, certificate domain, validator-set context, voting power and signatures. No Malachite certificate decoder or verifier is added.

The application adapter must perform canonical decoding, block-hash derivation, complete transaction validation and deterministic execution. Commit must be atomic, idempotent and reject stale parents; those guarantees are contractual and not enforced by this trait.

A crash between application commit and consensus acknowledgement is unresolved. Durable replay protection, cross-system transaction coordination, signer state and recovery are deferred to later phases.

## Consequences

- Protocol/application code has no Malachite type dependency.
- An engine can be replaced behind the verifier/adapter boundary without changing the application contract.
- Test doubles can verify orchestration ordering but cannot establish BFT correctness.
- A successful verifier return is a critical trust boundary and must not be supplied by an untrusted engine event or a boolean assertion.
- No production consensus engine, signer, network, finality, or atomic persistence is enabled.

## References

- Phase 20B isolated Malachite spike: `docs/protocol/phase-20b-malachite-isolated-api-spike.md`
- Phase 20C boundary record: `docs/protocol/phase-20c-application-engine-boundary.md`
- Consensus evaluation: `docs/protocol/consensus-engine-evaluation.md`
