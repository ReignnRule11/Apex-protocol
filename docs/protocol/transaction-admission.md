# Transaction Admission and Transition Planning — Phase 10

**Status:** static preflight, state-aware admission, pure transaction/block planning, versioned application, and a durable redb reference adapter are implemented as Rust prototypes. This is not full execution or production storage.

## Existing planning boundary

The `apex-consensus` crate performs chain/version/gas/fee-envelope checks, state-aware nonce and affordability checks, and pure transaction/block transition planning. Block order is preserved; sequential nonce/balance effects are applied through an overlay; recipient writes are merged; declared gas limits are accumulated; duplicate temporary IDs are rejected. Sender and transaction IDs are not cryptographically authenticated. Maximum fee is an escrow upper bound, not settled gas cost.

## Versioned state and persistent adapter

The `apex-state` crate defines versioned snapshots, plans bound to the exact parent StateVersion, and a backend-neutral fallible `StateStore` contract. Snapshot acquisition is fallible so backend failures are surfaced rather than converted to empty state or panics. The in-memory adapter remains a process-local reference implementation.

The separate `apex-state-redb` crate provides a contained redb 4.5.0 adapter prototype. It stores account records and the state version in one database and applies all account updates plus the next version in one immediate-durability write transaction. Snapshots are materialized inside one read transaction. Stale plans and before-state mismatches are rejected before writes. Close/reopen persistence, bootstrap non-overwrite, and integrity-check tests are included.

The adapter uses an internal 24-byte little-endian account record (u64 nonce + u128 balance), explicitly versioned as private storage schema. This is not protocol encoding. redb default exclusive-file locking is used; multi-process access is not enabled.

## Non-goals

No canonical transaction encoding/identity, signature verification, EVM execution, actual gas measurement, receipts, fee settlement/refunds, state root, Merkle proof, block-height binding, durable transaction index, consensus certificate verification or finality is implemented. The redb adapter has no process-kill/power-loss harness, disk-full injection, migration interruption test, backup/restore workflow, or production approval. A commit I/O error may be indeterminate; the caller must close/reopen and reconcile the persisted version before retrying.

## Validation gates

Add process interruption and fault-injection tests, cross-process concurrency tests, backup/restore and schema-migration tests, dependency audit, soak tests and independent storage review before production use. State-root specification remains a separate protocol decision.
