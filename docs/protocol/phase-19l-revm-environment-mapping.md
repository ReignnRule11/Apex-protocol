# Phase 19L — Explicit APEX-to-revm Environment Mapping

**Status:** Merged in PR #34; post-merge Rust CI passed.
**Release impact:** G0 and G1 remain BLOCKED.

## Objective

Translate a validated `BlockExecutionContext` into the pinned revm 43.0.3 configuration and block environment without revm mainnet defaults, host-derived values, silent integer truncation, or implicit fork selection.

## Implementation

- Add `apex-evm::revm_environment`.
- Require an explicit `RevmForkBinding` pairing the active APEX `ForkId` with a revm `SpecId`. A mismatched binding is rejected.
- Set `CfgEnv`'s chain ID from APEX input and explicitly set the selected `SpecId` and its revm mainnet gas parameter table. This gas table is a provisional Ethereum-semantics candidate, not an approved APEX gas policy.
- Populate block number, timestamp, gas limit, base fee, beneficiary and prevrandao from the immutable APEX context.
- Reject 256-bit chain IDs or base fees that do not fit revm 43.0.3's respective `u64` fields. Never truncate.
- Support only post-Merge, pre-Cancun `SpecId` values in this first adapter. Earlier profiles need an explicit difficulty input. Cancun and later profiles need blob-gas inputs; Amsterdam additionally needs slot semantics. Unsupported profiles fail closed.
- Clear revm's default blob environment rather than accidentally inheriting a fabricated zero blob context.
- No APEX chain ID, fork profile, activation height, gas table or production fork schedule is selected.

## Tests

- Field-by-field environment mapping with an explicit Shanghai binding.
- Fork-ID binding mismatch rejection.
- Wide chain-ID and base-fee overflow rejection.
- Pre-Merge and Cancun-or-later profile rejection.
- Missing post-Merge prevrandao rejection.

## Trust boundaries and limitations

The caller must supply a context derived from authenticated/validated block and genesis data, and a fork binding from protocol configuration. This adapter does not authenticate either source.

The adapter's revm gas parameters are Ethereum mainnet parameters for the explicitly supplied `SpecId`; APEX-specific gas schedules, precompiles, system contracts and fork semantics remain unratified. This is not evidence that an APEX fork is equivalent to an Ethereum fork.

The 256-bit APEX chain ID and base fee types are wider than revm 43.0.3's `u64` environment fields. This adapter therefore accepts only values representable without loss. A future engine/API decision is required before APEX can support larger values end-to-end.

Blob-gas context, Amsterdam slot semantics, pre-Merge difficulty, complete transaction environment, authenticated transaction decoding, fee settlement, receipts and full state commitment remain outside this phase. No transaction is executed here.

## Exit criteria

1. Ratify APEX fork semantics and the mapping to an exact revm `SpecId` for each activation.
2. Resolve the `u64` chain-ID/base-fee representation mismatch or formally constrain protocol values.
3. Add validated blob-gas and any future fork-specific block fields before enabling Cancun+ profiles.
4. Connect this environment to the actual EVM builder and transaction pipeline without `Context::mainnet()` defaults.
5. Add official execution-spec fixtures and differential tests across every supported fork boundary.

**Decision: NO-GO.** This phase creates an explicit environment-conversion boundary only; it does not make APEX execution production-ready.
