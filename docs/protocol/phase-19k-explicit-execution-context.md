# Phase 19K — Explicit EVM Block Execution Context

**Status:** PR #33; CI pending.  
**Release impact:** G0 and G1 remain BLOCKED.

## Objective

Remove implicit execution-environment inputs from the EVM integration boundary. Every execution must be reproducible from validated protocol configuration and a validated block context, without consulting host time, host randomness or Ethereum-mainnet defaults.

## Implementation

- Add APEX-owned `execution_context` module and depend on shared `apex-types::ChainId256`.
- `ForkId` is an opaque, fixed-width 32-byte profile identifier. It does not select a revm fork by itself.
- `ForkActivationSchedule` validates nonempty schedules, genesis activation at height zero, nonzero fork identifiers and strictly increasing activation heights.
- `active_at(height)` selects the latest activation at or before the requested height. Exact boundary behavior is tested.
- `BlockExecutionContext` is immutable from callers: private fields with read-only accessors for chain ID, fork ID, block number, timestamp, gas limit, 256-bit base fee, beneficiary and optional prevrandao.
- Construction rejects zero chain IDs, zero gas limits and missing prevrandao when the active profile requires it.
- No default chain ID, fork ID, activation height, timestamp, beneficiary or randomness is assigned.

## Tests

- Empty schedule and non-genesis first activation rejected.
- Zero fork ID and duplicate/non-increasing activation heights rejected.
- Fork selection tested immediately before, at and after activation boundaries.
- Explicit context fields preserved.
- Zero chain ID and zero gas limit rejected.
- Required prevrandao rejected when absent and optional before its configured activation.

## Trust boundaries and limitations

This module validates shape and internal schedule consistency; it does not authenticate genesis or the block proposal that supplied the values. The caller must pass data from a separately validated block/configuration path.

The schedule is configuration data, not a ratified APEX fork policy. A 32-byte fork ID is opaque. The `requires_prevrandao` boolean only controls presence of that field; it does not specify all fork-dependent opcode, precompile, header or system-contract semantics.

No revm `CfgEnv` or `BlockEnv` is constructed in this phase. No transaction is executed. No timestamp progression, base-fee recurrence, gas-limit adjustment, chain ID assignment, beneficiary policy or fork activation governance is ratified. No canonical serialization of the context is defined.

## Exit criteria

1. Ratify a fork profile and complete activation schedule with explicit fork-specific semantics.
2. Validate block context against authenticated parent/header and protocol transition rules.
3. Map the validated context losslessly into the pinned revm configuration/environment types.
4. Add boundary tests for every fork transition and official execution-spec fixtures.
5. Integrate authenticated type-2 transaction decoding, sender recovery, fee accounting, receipts and full-state commitment.

**Decision: NO-GO.** Phase 19K supplies the explicit context data boundary only; G0 and G1 remain blocked.
