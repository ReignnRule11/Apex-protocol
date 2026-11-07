# Execution Design Notes

The proposed execution environment is EVM-compatible, backed by a maintained Rust implementation such as revm subject to an ADR and pinned revision. No EVM execution is implemented yet.

## Pre-execution and planning boundary

The `apex-consensus` crate provides static envelope checks, state-aware nonce and affordability checks, pure single-transaction planning, and ordered block planning against an in-memory overlay. It preserves transaction order, merges account updates, and rejects the entire plan if any transaction fails. Temporary transaction IDs are not canonical hashes. Declared gas limits are accumulated; actual gas is not measured.

## Phase 8: versioned state application

The separate `apex-state` crate defines versioned snapshots and a compare-and-swap `StateStore` contract. A block plan is bound to the snapshot version from which it was derived. The in-memory reference adapter checks version equality, unique sorted account updates and every before-state value before applying any after-state; it increments the version exactly once. Stale/replayed plans and failed preconditions are rejected without mutation.

This is a process-local reference implementation, not durable storage. Production storage must provide consistent snapshots and atomically commit the state-version update plus every account write, with crash recovery and concurrency control. StateVersion is not a block height or state root. No EVM execution or authenticated transaction validation is implied.

Before execution implementation define account/storage model, signing domain, nonce rules, intrinsic gas, gas schedule, actual fee settlement, contract creation, precompiles, fork activation, receipt encoding, state-root algorithm and error semantics. Demonstrate compatibility through upstream vectors, APEX-specific vectors, differential testing and fuzzing. Host time, randomness, network access and nondeterministic iteration must not affect execution.
