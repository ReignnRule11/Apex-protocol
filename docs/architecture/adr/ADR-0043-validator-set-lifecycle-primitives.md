# ADR-0043: Deterministic Validator-Set Lifecycle Primitives

**Status:** Accepted for prototype only; production validator lifecycle NOT approved.
**Date:** 2026-11-07

## Context

Phase 20C introduced an engine-neutral application/finality handoff and Phase 20D introduced durable signing reservations. Neither defines a deterministic APEX validator-set representation or explicit activation boundary. Quorum arithmetic must not be confused with signature or certificate authentication.

## Decision

Add engine-neutral validator-set primitives to `apex-consensus`:

- Validator identities are opaque fixed-width 32-byte values; all-zero is invalid.
- Each validator has nonzero `u64` voting power.
- A set is nonempty, duplicate-free, sorted by identity, and its aggregate voting power must fit in `u64`.
- Quorum accounting accepts a list of validator identities, rejects duplicates and unknown identities, and requires strictly more than two-thirds of total power using `u128` intermediate arithmetic.
- A local schedule consists of complete replacement sets at strictly increasing, nonzero activation heights. The replacement applies at the exact activation height.
- An equivocation claim is an unverified typed envelope containing two distinct message digests for one signer/height/round/step coordinate.

## Deliberate non-decisions

This ADR does not ratify validator identity/key binding, consensus signature suite, canonical codec, validator-set hash, update authority, staking/bonding, activation delay, exit/unbonding, slashing, governance, certificate format or finality semantics. It does not authenticate updates or verify signatures.

## Consequences and residual risks

Canonical ordering and checked arithmetic reduce ambiguity in local validator-set handling. Exact-height selection makes activation semantics testable. However, local schedules are not authenticated governance, and signer identity values are not public-key validation. The quorum helper can only count supplied identities; it cannot establish that a certificate is valid. The evidence envelope cannot prove equivocation without authenticated signed messages and protocol-defined conflict rules.

Production consensus remains blocked on ratified protocol decisions, canonical encoding, authenticated validator updates, signature/certificate verification, engine integration, durable recovery, adversarial multi-process testing and independent security review.

## References

- Phase 20C: `docs/protocol/phase-20c-application-engine-boundary.md`
- Phase 20D: `docs/protocol/phase-20d-durable-signer-safety.md`
- Phase 20E: `docs/protocol/phase-20e-validator-lifecycle.md`
- ADR-0006: cryptographic domains
