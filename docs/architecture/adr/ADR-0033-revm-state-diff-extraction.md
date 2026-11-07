# ADR-0033: revm State-Diff Extraction Boundary

**Status:** Proposed pending Phase 19I CI and review  
**Date:** 2026-11-07

## Context

APEX has a revm 43.0.3 execution candidate, a read-only Database adapter over immutable full-width snapshots, APEX-owned state-delta types, and in-memory/persistent application boundaries. The missing integration seam is a deterministic conversion from revm's execution state map into the APEX delta representation.

Directly persisting revm-owned types would couple storage and protocol semantics to an upstream engine's internal representation. Directly mutating APEX state during execution would bypass atomic validation and make failed execution handling harder to reason about.

## Decision

Introduce an APEX-owned extraction adapter in `apex-evm::state_diff`:

- Consume revm `EvmState` plus the exact immutable parent snapshot.
- Emit deterministic, sorted APEX account, code and storage updates.
- Bind the delta to the parent snapshot's internal StateVersion.
- Validate bytecode hash/content consistency and each changed slot's original value against the parent.
- Leave publication to the existing APEX atomic state-application boundary.
- Reject selfdestruct/account deletion until the EVM fork and its deletion semantics are ratified.

## Alternatives considered

1. **Persist revm state directly.** Rejected because it leaks engine-specific state types into APEX storage and protocol boundaries.
2. **Mutate APEX state from inside the revm adapter.** Rejected because it bypasses the existing delta validation and atomic publication boundary.
3. **Implement account deletion immediately.** Rejected because account clearing and selfdestruct behavior depend on the selected EVM fork, which is not ratified.
4. **Wait until the complete transaction pipeline exists.** Rejected for this bounded integration step; extraction can be implemented independently while preserving explicit limitations.

## Consequences

- APEX owns the conversion and ordering contract.
- A stale or mismatched parent is still rejected by the application layer.
- Unsupported code/storage mismatches fail closed.
- The extraction function itself does not prove the transaction succeeded. Callers must pass only successful execution state; a success-gated API remains required.
- This is not canonical encoding, a state root, a fee/receipt pipeline, consensus integration, or production readiness.
- Tests using constructed revm state values validate adapter behavior only; official fixtures and differential tests remain required.
