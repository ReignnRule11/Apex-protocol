# ADR-0013: redb Durable State Adapter Spike

- **Status:** Accepted for contained prototype evaluation; not production approval
- **Date:** 2026-11-07

## Context

Phase 9 established a backend-independent StateStore contract. The redb 4.5.0 release documents ACID transactions, MVCC readers, crash-safe commits and an immediate durability mode. Its upstream manifest declares Rust 1.90, above APEX's previous 1.85 MSRV. SQLite is a mature alternative, but its WAL mode requires managing companion files/checkpoints and an exact SQLite version/durability configuration.

## Decision

1. Run a contained redb 4.5.0 adapter spike in a separate `apex-state-redb` crate.
2. Raise the workspace MSRV to 1.90 explicitly and pin CI/toolchain to 1.90.0.
3. Use redb's default exclusive-file locking and default `Durability::Immediate`; do not enable multi-process modes in this spike.
4. Persist the state version and all account updates in one redb write transaction.
5. Use a private, explicitly versioned 24-byte little-endian account record (u64 nonce + u128 balance). This is an internal storage schema only, not protocol encoding.
6. Keep `StateStore` backend-neutral. Snapshot failures are surfaced as errors rather than panics/default state.
7. Keep canonical state commitments, EVM state, block hashes, and durable consensus signer state out of this adapter.

## Consequences and limitations

The adapter is a single-process/single-handle prototype. It does not prove multi-process writer correctness, cross-platform power-loss behavior, fault-injection resilience, database migration safety, or production performance. redb's checksums are not APEX state commitments. The very recent 4.5.0 release needs sustained evaluation and upgrade review. No production validator should rely on this prototype without independent review, adversarial testing, and operational rehearsal.
