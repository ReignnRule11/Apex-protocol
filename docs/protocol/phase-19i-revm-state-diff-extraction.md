# Phase 19I — revm State-Diff Extraction

**Status:** implementation PR #31; CI and review pending.  
**Release impact:** G1 remains BLOCKED.

## Objective

Translate revm's execution state into APEX-owned, deterministic state deltas without bypassing the existing parent-version, before-state, ordering, snapshot-validation or atomic-apply boundaries.

## Implementation

- New module: `crates/apex-evm/src/state_diff.rs`.
- Public adapter: `extract_revm_state_delta(parent, state)`.
- Consumes revm's `EvmState` and emits APEX-owned account, code and storage updates.
- Uses ordered maps while extracting so output updates are deterministic regardless of revm's map iteration order.
- Binds the delta to the exact `EvmStateSnapshot::parent_state_version()`.
- Emits changed account information and changed storage slots; unchanged slots are omitted.
- Verifies provided bytecode against revm's declared code hash and rejects conflicting bytes for an existing content hash.
- Verifies each changed slot's revm original value against the parent snapshot before creating the update.
- Normalizes a resulting zero storage value to an absent slot.
- Does not publish state. The caller must pass the resulting delta to the existing atomic application adapter.

## Fail-closed boundary

Selfdestruct/account deletion is rejected. APEX has not ratified an EVM fork or activation schedule, and deletion semantics are fork-sensitive. The adapter must not guess whether an account is deleted or retained.

The function accepts `EvmState`, not a success-gated transaction result wrapper. Therefore, the caller is responsible for passing only state from a successful execution result. Reverted and halted transactions must never be applied. A future phase must make this condition structural in the API.

## Tests

- Extract a storage change from a constructed revm state fixture, apply the resulting parent-bound delta, and verify the resulting APEX snapshot.
- Reject a storage diff whose original value does not match the selected parent.
- Reject selfdestruct/account deletion while fork semantics remain unresolved.

These are adapter-level tests. They are not official Ethereum execution-spec fixtures, cross-client differential tests, a full transaction-to-persistent-state integration test, or an independent audit.

## Explicitly out of scope

- Transaction decoding, sender recovery, signature verification or replay protection.
- Success-gated execution result handling.
- Fork/block context and fork activation.
- EIP-1559 escrow, gas settlement, burn, priority-fee credit and refunds.
- Receipts, logs, transaction and receipt roots, full state commitment.
- Consensus/signer/application atomic commit.
- Production database selection, multi-process operation or power-loss assurance.

## Exit criteria

1. Success/revert/halt outcome is structurally bound to state extraction.
2. Explicit validated block context and ratified fork semantics are supplied.
3. Account creation, code replacement, storage writes, deletions and edge cases have property and adversarial tests.
4. Official fork-specific execution fixtures and independent-client differential tests pass.
5. Resulting deltas replay deterministically and feed canonical receipts, fees and complete state commitments.

**Decision: NO-GO.** Phase 19I does not close G1 or authorize production execution.
