# Phase 13 — Storage Lifecycle and Schema Migration

**Status:** implementation submitted for review; CI pending. Not production-approved.

## Implementation

- Schema 1: 24-byte account records (u64 nonce + u128 balance).
- Schema 2: separate `apex_accounts_v2` table, 32-byte records with the same fields and an eight-byte zero-reserved field. Reserved bytes must remain zero and have no protocol meaning.
- Explicit `migrate_v1_to_v2` validates all records, copies to v2, deletes v1 and updates metadata in one redb write transaction. StateVersion is preserved. Normal open rejects schema 1; operators must migrate while the database is not concurrently open.
- Repeated migration on schema 2 is idempotent. Unknown schemas and malformed legacy records fail closed.

## Lifecycle

The adapter starts Ready. If redb commit returns an error, it becomes ReopenRequired and rejects subsequent snapshots and writes. The operator must close/reopen, let redb recover, and reconcile persisted StateVersion and state before deciding whether to retry. No automatic retry or journal is included.

## Locking decision

Retain redb's default ExclusiveWriter mode. Upstream redb 4.5.0 documents SingleWriter and MultiWriter modes, but APEX does not enable them. Independent-process locking, crash/reopen and contention tests are required before changing this decision.

## Tests and acceptance

- [x] Atomic schema 1 to 2 migration implementation.
- [x] Preserve StateVersion, nonce and balance.
- [x] Idempotent migration and malformed-record rollback tests.
- [x] Reopen-required latch after commit error.
- [x] Document unenabled process modes and limitations.
- [ ] Format, Clippy and workspace CI pass.
- [ ] Independent-process locking tests and cross-platform validation.
- [ ] Real storage I/O fault injection and operational migration runbook.

This is a research prototype. No rollback-to-v1, online migration, power-loss simulation, multi-process mode or production recovery journal is included.
