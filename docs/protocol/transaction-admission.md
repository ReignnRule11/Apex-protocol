# Transaction Admission and Transition Planning — Phase 8

**Status:** static preflight, state-aware admission, pure transaction/block planning, and a versioned state-application reference boundary are implemented as Rust prototypes. This is not full execution or durable storage.

## Existing planning boundary

The `apex-consensus` crate performs chain/version/gas/fee-envelope checks, state-aware nonce and affordability checks, and pure transaction/block transition planning. Block order is preserved; sequential nonce/balance effects are applied through an overlay; recipient writes are merged; declared gas limits are accumulated; duplicate temporary IDs are rejected. Sender and transaction IDs are not cryptographically authenticated. Maximum fee is an escrow upper bound, not settled gas cost.

## Phase 8: versioned snapshots and atomic application

The `apex-state` crate adds:

- `StateVersion(u64)`: a monotonic internal concurrency token, not block height or state root.
- `VersionedStateSnapshot`: an immutable-by-interface account snapshot implementing `AdmissionState`.
- `VersionedBlockTransitionPlan`: a block plan bound to the exact parent snapshot version.
- `StateStore`: a contract requiring consistent snapshots and linearizable compare-and-swap application.
- `InMemoryStateStore`: a process-local reference adapter for tests.

The adapter checks the plan's parent version, strictly increasing unique account-update keys, and every before-state value before the first write. It then publishes every after-state and increments the version once. Stale/replayed plans, mismatched before-state, malformed update order, and version overflow fail without mutation.

This is not durable, crash-safe or multi-process storage. A production adapter must atomically commit the version and all account writes in one serializable transaction or equivalent, with consistent snapshot reads, concurrent-writer handling and crash recovery.

## Non-goals

No canonical transaction encoding/identity, signature verification, EVM execution, actual gas measurement, receipts, fee settlement/refunds, state root, block height binding, durable transaction index, production database schema, recovery/journaling, consensus certificate verification or finality is implemented. The state adapter checks preconditions; it does not prove that a plan came from trusted execution.

## Validation gates

Add property-based and fuzz testing, concurrency and crash-injection tests, durable database adapter tests, state-root specification and independent review before production use.
