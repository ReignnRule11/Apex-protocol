# ADR-0034: Success-Gated revm State Extraction

**Status:** Proposed pending review; Phase 19J PR CI #42391906701 passed  
**Date:** 2026-11-07

## Context

Phase 19I introduced an adapter accepting a raw revm `EvmState`. Its documentation required callers to pass only a successful execution's state, but that requirement was a convention rather than an API-enforced boundary. revm returns an execution outcome and state together; a `Revert` or `Halt` must not be converted into committed APEX state.

## Decision

Add `extract_successful_revm_result_delta` over revm's paired `ExecResultAndState<ExecutionResult, EvmState>` value:

- Match the execution outcome before reading the state.
- Delegate only `ExecutionResult::Success` to Phase 19I extraction.
- Reject `Revert` and `Halt` with distinct errors.
- Preserve parent-version binding and the existing APEX atomic apply boundary.

## Consequences

- Revert and halt outcomes cannot pass through the recommended extraction API.
- The result/state pair is structurally bound, but public revm types can be constructed by callers; provenance must be owned by the block execution pipeline.
- EVM success is not transaction authorization or consensus inclusion.
- This does not implement transaction authentication, fork/block context, fee settlement, receipts, state roots or consensus commit.
- Fork-sensitive selfdestruct remains fail-closed.

## Alternatives considered

1. **Continue with a caller convention.** Rejected because it permits accidental extraction from revert/halt state.
2. **Accept only raw state and a boolean.** Rejected because a boolean can be mismatched with an unrelated state map.
3. **Consume revm's paired result/state value.** Selected because it preserves the engine's outcome/state association and allows an exhaustive match over success, revert and halt.
