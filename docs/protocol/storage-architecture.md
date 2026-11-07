# Persistent State Storage Architecture — Phases 10–13

**Status:** redb adapter prototype; Phase 13 migration/lifecycle implementation under review; not production-approved.

## Schema 2

Schema 1 used `apex_accounts_v1` with 24-byte records (little-endian u64 nonce + u128 balance). Schema 2 uses `apex_accounts_v2` with 32-byte records: the same fields plus an eight-byte zero-reserved field. Nonzero reserved bytes are rejected; the field has no protocol meaning. AccountId remains an internal u64 placeholder. This is not canonical encoding or a state commitment.

Migration is explicit through `migrate_v1_to_v2`. It validates metadata and every legacy record, copies values, deletes the old table and updates the schema marker in one redb write transaction. StateVersion is preserved. Malformed records and unknown schema versions fail closed. Re-running on schema 2 is idempotent. Normal open does not silently migrate; operators must ensure exclusive operational ownership, migrate, then reopen.

## Lifecycle and atomic application

Block application validates schema/version and all preconditions before mutation; account writes and StateVersion update occur in one Immediate-durability transaction. If commit returns an error, the adapter latches ReopenRequired and rejects further snapshots and writes. Close and reopen, allow redb recovery, then reconcile persisted StateVersion and account state before deciding whether retry is safe. There is no automatic retry or operation journal.

## Concurrency

The adapter retains redb 4.5.0's default ExclusiveWriter mode. SingleWriter and MultiWriter exist upstream but remain disabled and unvalidated by APEX. The StateStore API still has `snapshot(&self)` and `apply(&mut self)`; existing tests do not establish reader/writer overlap through that API. Multi-process lock tests remain outstanding.

## Evidence and gaps

Existing tests cover state application, stale-plan rejection, process-abort recovery, cold backup, concurrent readers and mutex-serialized writers. Phase 13 adds migration preservation/idempotency and malformed-record rollback tests. Not covered: independent-process contention, actual disk-full/device I/O injection, physical power loss, online backup, operation journal, soak tests, benchmarks or independent review. No production-readiness claim is made.
