# ADR-0014: Storage Reliability Validation Boundaries

- **Status:** Accepted for prototype test coverage; production approval withheld
- **Date:** 2026-11-07

## Context

Phase 10 introduced a persistent redb 4.5.0 adapter with atomic account/version commits. Normal close/reopen and precondition-failure tests do not exercise unclean process termination, backup recovery, or unsupported schema behavior. redb documents crash recovery on open, MVCC readers, and a single in-progress writer per database handle. Its integrity check is an explicit repair/check operation and cannot run while transactions remain active on that handle.

## Decision

1. Add subprocess-abort tests for both an uncommitted transaction and a transaction whose commit() returned success.
2. Reopen the same database after each child process terminates and assert the committed version/account state.
3. Test snapshot stability by retaining a materialized snapshot across a subsequent commit.
4. Test only a cold backup: close the database, copy the file, reopen the copy, and compare the logical snapshot.
5. Test that an unknown APEX metadata schema version fails closed.
6. Treat any commit() error as indeterminate at the APEX boundary. Do not retry automatically; close/reopen and reconcile persisted state first.
7. Keep power-loss, kernel crash, disk-full, injected I/O errors, migration interruption, cross-process access and production backup orchestration explicitly out of scope until separately tested.

## Consequences

The new tests improve evidence for process-level recovery and controlled cold restore. They do not prove durability under physical power loss, kernel failure, storage-controller cache loss, or all filesystem/device combinations. The cold-copy test is not an online backup facility. No multi-process concurrency mode is enabled, and no schema migration framework is introduced by this ADR.

## References

- redb 4.5.0 Database API: https://docs.rs/redb/4.5.0/redb/struct.Database.html
- redb 4.5.0 changelog: https://docs.rs/crate/redb/4.5.0/source/CHANGELOG.md
