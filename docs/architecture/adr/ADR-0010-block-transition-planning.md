# ADR-0010: Ordered Block Transition Planning

- **Status:** Accepted for prototype implementation
- **Date:** 2026-11-07
- **Decision owners:** APEX protocol maintainers

## Context

Phase 6 provided a pure single-transaction transition planner. It could not model a block containing multiple transactions from the same sender, cumulative declared gas limits, or multiple writes to the same recipient. A block-level planning boundary is required before defining durable state application.

## Decision

Implement `plan_block_transition` as a pure ordered planner over a caller-supplied parent-state view.

1. Preserve caller-provided transaction order; do not sort transactions.
2. Use an in-memory account overlay so each transaction observes preceding planned nonce and balance changes.
3. Merge all writes to each account and emit one final before/after update per touched account, sorted by internal AccountId.
4. Reject duplicate caller-supplied `TransactionId(u64)` values within one block. This identifier is temporary and opaque; it is not a cryptographic hash or canonical transaction identity.
5. Accumulate declared transaction gas limits with checked arithmetic and reject totals above the caller-supplied block gas limit.
6. Return an error and no plan if any transaction fails. Never mutate the supplied parent state.
7. Accumulate maximum-fee escrow with checked arithmetic, while continuing to treat it as a conservative upper bound rather than a settled fee.

## Consequences and limitations

The planner supplies deterministic in-memory accounting for the currently modeled transfer/nonce subset. It does not execute EVM code, authenticate transaction senders or IDs, calculate actual gas, settle fees, create receipts or state roots, persist state, enforce snapshot isolation, or establish consensus finality. The application layer must eventually bind the plan to an immutable parent-state version and commit all updates atomically or none.

The caller-provided block gas limit is not a ratified network parameter. No protocol-level transaction ordering policy is selected by this ADR; the planner's contract is simply to preserve the supplied order.

## Validation

Unit tests cover sequential same-sender transactions, merged recipient credits, duplicate ID rejection, cumulative gas-limit rejection, late failure without parent mutation, and deterministic output for identical ordered inputs. These tests are not a substitute for property-based, fuzz, persistence, cross-process replay or independent security testing.
