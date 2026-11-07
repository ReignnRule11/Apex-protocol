# ADR-0037: Build revm from Explicit APEX Inputs

**Status:** Proposed pending PR validation and review
**Date:** 2026-11-07

## Context

Phase 19L maps a validated APEX block execution context and explicit fork binding into revm 43.0.3 `CfgEnv` and `BlockEnv`. The mapping must now reach the actual EVM builder without being replaced by revm defaults. APEX also has an immutable full-width state snapshot and a read-only revm `Database` adapter.

## Decision

Provide an APEX-owned `build_revm_evm` constructor accepting:
- an immutable `EvmStateSnapshot`;
- a `RevmExecutionEnvironment` previously created by the Phase 19L fail-closed mapper.

The constructor uses the snapshot's `EvmDatabaseView`, creates a revm context with that database and the explicit `SpecId`, then installs the exact mapped `CfgEnv` and `BlockEnv` before calling revm's `MainBuilder::build_mainnet`.

The name `build_mainnet` is revm's API for selecting its standard Ethereum instruction and precompile implementation. It is not permission to create a mainnet context or select a default fork. The implementation must not call `Context::mainnet()`, use `SpecId::LATEST`, or reconstruct any environment field from host/default values.

## Consequences

- The actual EVM instance is tied to the caller's immutable APEX snapshot and explicit mapped block/fork environment.
- The builder remains an Ethereum instruction/precompile implementation candidate, not a ratified APEX execution specification.
- The constructor does not authenticate block/configuration inputs; callers remain responsible for that trust boundary.
- No transaction decoding/authentication, transaction execution, fee settlement, receipt construction or persistent state commit is added.
- G0/G1 remain blocked.

## Alternatives considered

1. **Call `Context::mainnet()` then overwrite selected fields.** Rejected because unrepresented fields and future defaults could leak into execution.
2. **Use `SpecId::LATEST`.** Rejected because fork behavior would change independently of APEX protocol configuration.
3. **Build from an empty/default database.** Rejected because execution must read the exact parent snapshot.
4. **Construct from the APEX snapshot and explicit environment.** Selected as the narrowest auditable builder boundary.
