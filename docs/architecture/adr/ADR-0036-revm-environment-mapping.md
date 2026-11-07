# ADR-0036: Explicit revm Environment Mapping

**Status:** Implemented (merged); APEX fork mapping remains unratified
**Date:** 2026-11-07

## Context

Phase 19K introduced an immutable APEX block execution context and an explicit fork schedule. The context must be translated into revm 43.0.3 inputs without calling `Context::mainnet()`, relying on a default/latest fork, or silently narrowing APEX's 256-bit numeric types.

## Decision

Add an APEX-owned `RevmExecutionEnvironment` constructor requiring both a validated `BlockExecutionContext` and an explicit `RevmForkBinding`.

The binding pairs the context's active opaque `ForkId` with a caller-supplied revm `SpecId`. The pair must match exactly. The adapter explicitly sets the revm `CfgEnv` spec, gas parameter table and chain ID, then copies block fields into `BlockEnv`.

The first supported profile range is post-Merge and pre-Cancun:
- Pre-Merge is rejected because the current APEX context does not carry proof-of-work difficulty.
- Cancun and later are rejected because the current context has no blob-gas inputs.
- Amsterdam and later are additionally outside the current profile because slot semantics are not represented.
- Post-Merge contexts must contain prevrandao.

The adapter rejects chain IDs and base fees that exceed revm 43.0.3's `u64` fields. It never truncates. It explicitly clears revm's default blob environment.

## Consequences

- No fork is selected implicitly; caller configuration must bind an APEX ForkId to a revm SpecId.
- The adapter uses revm's Ethereum mainnet gas parameter table for that SpecId as a provisional compatibility mapping. This is not ratified APEX gas policy.
- APEX 256-bit chain-ID/base-fee representations cannot yet use their full range through this revm adapter.
- Unsupported block fields/forks fail closed until the context model and adapter are extended.
- The mapping is not an authenticated block validator, transaction pipeline, or proof of execution conformance.

## Alternatives considered

1. **Use `Context::mainnet()` or `SpecId::LATEST`.** Rejected because this silently chooses defaults and can change across revm releases.
2. **Truncate 256-bit values to u64.** Rejected because it creates collisions and divergent execution inputs.
3. **Accept every SpecId and leave absent fields at revm defaults.** Rejected because missing difficulty/blob/slot values would become fabricated execution inputs.
4. **Require an explicit binding and support only the representable context subset.** Selected as the bounded fail-closed first adapter.
