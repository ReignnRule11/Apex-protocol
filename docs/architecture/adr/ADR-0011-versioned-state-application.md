# ADR-0011: Versioned Snapshots and Atomic State Application

- **Status:** Accepted for prototype implementation
- **Date:** 2026-11-07

## Context

Phase 7 produces a pure block transition plan but does not bind it to a particular parent-state version or define a safe application boundary. Applying a plan to changed state, replaying it, or partially writing account updates could corrupt application state.

## Decision

Create a separate `apex-state` crate with a monotonic internal `StateVersion`, immutable-by-interface `VersionedStateSnapshot`, version-bound `VersionedBlockTransitionPlan`, and `StateStore` compare-and-swap contract. Include an `InMemoryStateStore` reference adapter for tests.

An adapter must reject stale parent versions; validate strictly increasing unique account keys and every before-state value before the first write; then publish every after-state and increment the version exactly once. Any error must leave all accounts and the version unchanged. Reapplication of a successful plan is rejected as stale.

## Consequences

The in-memory adapter is process-local and non-durable. It is not crash-safe, multi-process, or production storage. A production adapter must atomically commit the version and every account write in a serializable transaction or equivalent, provide consistent snapshots, and test concurrent writers, crash recovery, journaling, corruption and backup/restore.

StateVersion is not block height, block hash or a cryptographic state root. The adapter checks state preconditions; it does not authenticate transactions, validate consensus certificates, prove plan provenance, execute EVM code or compute commitments.
