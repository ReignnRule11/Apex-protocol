# Execution Design Notes

The proposed execution environment is EVM-compatible, backed by a maintained Rust implementation such as revm subject to an ADR and pinned revision. No EVM execution is implemented yet.

## Pre-execution and planning boundary

The `apex-consensus` crate provides static envelope checks, state-aware nonce and affordability checks, pure single-transaction planning, and ordered block planning against an in-memory overlay. It preserves transaction order, merges account updates, and rejects the entire plan if any transaction fails. Temporary transaction IDs are not canonical hashes. Declared gas limits are accumulated; actual gas is not measured.

## Phase 8: versioned state application

The `apex-state` crate defines versioned snapshots, version-bound plans and a backend-neutral fallible `StateStore` contract. Its in-memory adapter remains a process-local reference implementation. The separate `apex-state-redb` crate adds a persistent prototype using one immediate-durability transaction for all account writes and the next StateVersion. Snapshot materialization uses one read transaction; stale plans and mismatched before-state values are rejected before mutation. The adapter is currently single-process/single-handle.

Its 24-byte little-endian account representation is a private, versioned storage schema, not canonical protocol encoding. StateVersion is not a block height or state root. No EVM execution or authenticated transaction validation is implied.

## Phase 9: storage and commitment architecture

The storage design is specified in `storage-architecture.md`. A future durable adapter must atomically commit every account update and the state version, expose consistent snapshots, reject stale plans, and recover to a complete old or new committed state after interruption. Database integrity metadata is not a consensus commitment. No storage backend has been adopted.

The separate `state-commitments.md` document defines requirements for a deterministic authenticated state root. No root is implemented because canonical account encoding, key ordering, hash domains, and authenticated tree semantics remain open. A database migration must not change a root for identical logical state.

Before execution implementation define account/storage model, signing domain, nonce rules, intrinsic gas, gas schedule, actual fee settlement, contract creation, precompiles, fork activation, receipt encoding, state-root algorithm and error semantics. Demonstrate compatibility through upstream vectors, APEX-specific vectors, differential testing and fuzzing. Host time, randomness, network access and nondeterministic iteration must not affect execution.
