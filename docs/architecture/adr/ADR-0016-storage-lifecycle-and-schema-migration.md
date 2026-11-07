# ADR-0016: Storage Lifecycle and Schema Migration

- **Status:** Accepted for prototype implementation; production approval withheld
- **Date:** 2026-11-07

## Context

Schema evolution must preserve account state and must not expose partially migrated data. redb 4.5.0 documents that a commit error may have become durable and that further writes require close/reopen recovery.

## Decisions

1. Introduce schema 2 in a separate `apex_accounts_v2` table with a 32-byte record: u64 nonce, u128 balance and an eight-byte zero-reserved field.
2. Keep migration explicit; normal open rejects schema 1. Migration requires exclusive operational ownership.
3. Validate and copy records, delete the old table and update the schema marker in one redb write transaction.
4. Preserve StateVersion; unknown schemas and malformed records fail closed.
5. Latch the adapter ReopenRequired after commit errors; reject further access until reopen and state reconciliation.
6. Retain default ExclusiveWriter mode. Do not enable other process modes without independent-process tests.

## Consequences

Migration is atomic at the redb transaction boundary and one-way. Reserved bytes have no assigned meaning. The lifecycle latch does not infer commit outcome; reopen/reconciliation remains an operational responsibility. No automatic retry or operation journal exists.

## References

- https://docs.rs/crate/redb/latest/source/CHANGELOG.md
- https://docs.rs/redb/latest/redb/struct.WriteTransaction.html
- `docs/protocol/storage-architecture.md`
