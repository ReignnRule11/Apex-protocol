# Execution Design Notes

The proposed execution environment is EVM-compatible, backed by a maintained Rust implementation such as revm subject to an ADR and pinned revision. No EVM execution is implemented yet.

## Implemented pre-execution boundary

The `apex-consensus` crate provides static transaction-admission checks for chain ID, protocol version, nonzero gas limit, per-transaction gas bound, fee-cap ordering and overflow-safe maximum-fee arithmetic. State-aware admission checks sender existence, exact nonce and conservative balance sufficiency against a caller-supplied state view. These checks do not authenticate the sender or decode a canonical transaction.

## Pure transition planning

Phase 6 added a single-transaction planner. It computes sender nonce advancement, maximum-fee escrow debit and recipient value credit, returning before/after account values without mutating storage. The caller must commit all updates atomically or none.

## Phase 7: ordered block planner

The new `plan_block_transition` function processes caller-supplied transactions in their original order against an in-memory account overlay. Later transactions observe earlier nonce and balance changes. Repeated recipient credits are merged into one final update per account. Duplicate caller-supplied opaque transaction IDs are rejected. Declared gas limits are accumulated with checked arithmetic and compared with the caller-supplied block gas limit. The final update vector is sorted by internal AccountId for deterministic output; this does not reorder transactions.

The planner returns no plan if any transaction fails and does not mutate the parent state. It does not perform EVM execution, calculate actual gas, produce receipts, settle fees, compute state roots or persist state. TransactionId is a temporary local duplicate-detection key, not a transaction hash or cryptographic identity. A future application layer must pin the plan to an immutable parent-state version and atomically commit the complete resulting state.

Before execution implementation define account/storage model, signing domain, nonce rules, intrinsic gas, gas schedule, actual fee settlement, contract creation, precompiles, fork activation, receipt encoding, state-root algorithm and error semantics. Demonstrate compatibility through upstream vectors, APEX-specific vectors, differential testing and fuzzing. Host time, randomness, network access and nondeterministic iteration must not affect execution.
