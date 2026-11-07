# ADR-0035: Explicit EVM Block Execution Context

**Status:** Proposed pending PR #33 CI and review  
**Date:** 2026-11-07

## Context

The revm harness and state adapter do not yet receive a complete, validated APEX block execution context. Using implicit mainnet defaults, host time or process randomness would make execution configuration ambiguous and could cause validators to disagree at fork boundaries.

## Decision

Introduce an APEX-owned immutable `BlockExecutionContext` and validated `ForkActivationSchedule`.

The context explicitly carries:
- 256-bit APEX chain ID;
- opaque 32-byte active fork-profile ID;
- block number and timestamp;
- nonzero block gas limit;
- 256-bit base fee per gas;
- beneficiary/fee recipient address;
- optional 32-byte prevrandao, required only when the selected profile declares it mandatory.

The schedule must begin at height zero, use nonzero fork IDs and have strictly increasing activation heights. Selection is inclusive: an activation applies at its exact configured height.

## Consequences

- Execution inputs become explicit and immutable after construction.
- Host time, process randomness and implicit mainnet defaults are excluded from this context API.
- No chain ID, fork identity or activation height is selected by this ADR.
- The schedule must be supplied by a future ratified genesis/protocol configuration.
- The context is not authenticated by this crate and is not yet mapped into revm environment structures.
- The prevrandao requirement flag is not a complete fork semantics definition.
- G0/G1 remain blocked pending fork ratification, header validation, transaction authentication, revm environment mapping and conformance evidence.

## Alternatives considered

1. **Use revm mainnet defaults.** Rejected because APEX fork selection and activation are not ratified.
2. **Read timestamp/randomness from the host.** Rejected because consensus execution must be deterministic and block-derived.
3. **Use opaque configuration with no validation.** Rejected because empty, unordered or zero-ID fork schedules can produce ambiguous activation.
4. **Use typed context and validated activation schedule.** Selected as a bounded, fail-closed data model without pretending protocol decisions have been made.
