# Phase 19M — Explicit revm Builder Integration

**Status:** Implementation on branch `phase-19m/revm-builder-integration`; validation pending.
**Release impact:** G0 and G1 remain BLOCKED.

## Objective

Connect the explicit APEX-to-revm environment mapping from Phase 19L and the immutable APEX EVM state snapshot to the actual revm 43.0.3 execution builder. The builder must receive the exact mapped configuration and block environment; it must not create a default/mainnet context or select an implicit/latest fork.

## Implementation

- Add `apex-evm::revm_execution::build_revm_evm`.
- Accept an immutable `EvmStateSnapshot` and a previously validated `RevmExecutionEnvironment`.
- Use the snapshot's read-only `EvmDatabaseView` as the revm database.
- Extract the mapped `CfgEnv` and `BlockEnv`; create the revm context with the supplied database and selected `SpecId`; install the exact mapped config and block values before invoking the builder.
- Use revm's `MainBuilder::build_mainnet` only to select the standard Ethereum instruction/precompile implementation. This method does not call `Context::mainnet()`, create a mainnet context, or choose `SpecId::LATEST`. The fork is the explicit value already carried by `CfgEnv`.
- Add a builder-level test that inspects the constructed EVM's chain ID, SpecId, block number, timestamp, gas limit, base fee, beneficiary, prevrandao and parent state version.

## Scope boundaries

This phase constructs the actual EVM engine with explicit APEX inputs. It does not yet:
- Decode or authenticate a transaction.
- Populate a canonical `TxEnv` from a signed APEX envelope.
- Execute a transaction or block as a protocol operation.
- Settle gas fees, burn base fees, pay proposer priority fees or refund unused gas.
- Produce canonical receipts/log commitments.
- Convert successful execution output into and atomically persist a complete state transition.
- Authenticate the source block/genesis data or ratify the APEX-to-revm fork mapping.

The current environment adapter remains limited to post-Merge/pre-Cancun profiles and u64-representable chain IDs/base fees. Its Ethereum gas table is provisional, not APEX policy.

## Exit criteria

1. CI proves the builder compiles against the exact revm 43.0.3 pin.
2. Builder-level tests prove the supplied APEX environment and snapshot are installed unchanged.
3. No implicit context/fork defaults are used.
4. Follow-on transaction execution tests demonstrate success, revert and halt handling against fixed parent snapshots.
5. Official execution fixtures, differential tests, complete fee/receipt/state-root semantics and independent review remain required before G1 can pass.

**Decision: NO-GO.** This is builder wiring, not a complete or production-ready transaction pipeline.
