# Transaction Admission and Transition Planning — Phase 7

**Status:** static preflight, state-aware admission, pure single-transaction planning, and ordered in-memory block planning are implemented as Rust prototypes. This is not full transaction or block execution.

## Static and state-aware admission

`validate_transaction_admission` checks chain ID, protocol version, nonzero gas limit, per-transaction gas bound, priority-fee/max-fee ordering, and checked multiplication of gas limit by maximum fee per gas. It returns a maximum-fee upper bound, not a charged or settled fee.

`validate_transaction_against_state` reads the sender from a caller-supplied read-only state view, rejects a missing sender, requires exact nonce equality, and checks balance against value plus the maximum-fee upper bound. The sender identifier is not authenticated. The state provider must expose a stable view of the exact candidate parent state; this prototype does not enforce snapshot isolation or state-root binding.

## Phase 6: single-transaction planning

`plan_transaction_transition` advances the sender nonce, debits value plus maximum-fee escrow, and credits an existing recipient. Self-transfers net value to zero and deduct only the fee escrow. It returns before/after values and never mutates storage. A future storage adapter must commit every update atomically or none.

## Phase 7: ordered block planning

`plan_block_transition` accepts an ordered slice of `BlockTransaction` entries and constructs a pure in-memory overlay over the supplied parent state.

- Input order is execution order; the planner does not sort or reorder transactions.
- Each transaction sees the nonce and balances produced by all preceding transactions in the same candidate block. Multiple sequential transactions from one sender are therefore checked against the incremented overlay nonce and reduced overlay balance.
- Recipient credits are merged into one final account update rather than emitted as conflicting writes.
- Cumulative transaction gas limits are added with checked arithmetic and must not exceed the caller-supplied block gas limit. This is a sum of declared gas limits, not measured gas consumption.
- Duplicate `TransactionId` values are rejected within the candidate block. `TransactionId(u64)` is a temporary caller-supplied opaque identifier used only for local duplicate detection. It is not a cryptographic hash, signature, canonical transaction ID, or replay-protection mechanism.
- The final account-update vector is ordered by `AccountId` using a `BTreeMap`; this canonicalizes output ordering only. It does not change transaction execution order.
- If any transaction fails, the function returns an error and no plan. The parent state is never mutated. Aggregate maximum-fee escrow is also accumulated with checked arithmetic.

The result reports transaction count, cumulative declared gas limit, aggregate maximum-fee escrow, and one before/after update per touched account. Empty input yields an empty plan. No durable write occurs; the future application layer must bind the plan to a versioned parent-state snapshot and atomically commit all updates or none.

## Trust boundaries and non-goals

Not implemented: canonical transaction encoding or identifiers, signatures or sender derivation, intrinsic gas, EVM execution, transaction receipts, actual gas usage, fee settlement/refunds, base-fee burn, proposer rewards, state roots, contract storage, account creation, durable atomic commit, block header/certificate validation, consensus finality, or cross-node replay verification. Caller-provided transaction IDs and state views remain untrusted inputs. The planner is a deterministic arithmetic prototype, not an execution engine.

Zero fee remains permitted until fee policy is ratified. The block gas limit remains caller-supplied rather than an approved network parameter.

## Tests

Tests cover sequential same-sender nonces, merged recipient credits, duplicate opaque IDs, cumulative gas-limit rejection, late-transaction failure without parent mutation, and deterministic output for identical ordered inputs. These unit tests do not establish persistent-state correctness, cross-process determinism, signature validity, or execution correctness.

## Next gates

1. Ratify canonical encoding and chain/version-bound signing domain.
2. Select reviewed signature algorithms and add verification vectors.
3. Replace placeholder account and transaction identifiers with approved types.
4. Bind reads and writes to a versioned parent-state snapshot and atomic commit contract.
5. Specify intrinsic gas, actual gas accounting, fee settlement, receipts and state-root construction.
6. Add property-based, fuzz, persistence, fault-injection and cross-implementation replay tests.
7. Integrate a pinned execution engine only after the above semantics and compatibility requirements are reviewed.
