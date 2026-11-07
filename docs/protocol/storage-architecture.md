# Persistent State Storage Architecture — Phase 10

**Status:** redb adapter prototype implemented; not production-approved.

## Logical data model

The persistent prototype stores:
- Metadata table: schema version and monotonic StateVersion.
- Account table: prototype AccountId(u64) to 24-byte private account record (u64 nonce + u128 balance, little-endian).

The private record layout is a storage implementation detail. It is not the canonical protocol encoding, account-address format, or state-root input specification. Schema version is stored explicitly and unknown versions fail closed.

## Atomic block application

The redb adapter performs the following inside one immediate-durability write transaction:
1. Read and validate metadata schema and current state version.
2. Require the plan parent version to match.
3. Check version increment overflow.
4. Validate strict account-update ordering and every before-state.
5. Write every account after-state.
6. Write the next state version.
7. Commit once.

All account writes and the state-version update are within the same database transaction. A failed precondition drops the uncommitted transaction; a commit I/O error is treated as potentially indeterminate and must be reconciled by closing/reopening and inspecting the persisted version before retrying. The adapter never retries a failed commit automatically.

## Snapshot and concurrency contract

A snapshot is read inside one redb read transaction and materialized into a sorted BTreeMap. The snapshot contains one committed StateVersion. The adapter uses redb's default exclusive-file locking mode and is currently single-process/single-handle. It does not yet test concurrent independent writer processes or offer multi-process mode. A future multi-process adapter must explicitly select and test redb 4.5's concurrency mode.

## Recovery and operational requirements

- redb performs its documented recovery when the database is opened.
- The adapter exposes an explicit integrity-check method.
- Malformed metadata, unsupported schema versions, and malformed account records return errors; no defaults are synthesized.
- Backups must use a supported consistent mechanism or occur after closing the database. Copying a live database file is not supported by this prototype.
- Immediate durability is the redb default and is retained; the adapter does not request non-durable commits.
- No claim is made about power-loss behavior beyond the backend's documented contract and supported filesystem/platform assumptions.

## redb version and compatibility

The adapter pins redb 4.5.0 exactly. Its upstream Cargo manifest declares Rust 1.90 and MIT OR Apache-2.0 licensing. The 4.5.0 release is recent; its release notes describe multi-process concurrency modes and fixes to integrity checking and page-cache behavior. The workspace MSRV and CI toolchain are therefore explicitly raised to 1.90.0 for this contained evaluation.

## Conformance status

Implemented tests cover:
- Bootstrap and first initialization.
- Durable multi-account application and version increment.
- Close/reopen persistence.
- Stale-plan rejection.
- Late before-state mismatch with no partial writes.
- Existing database is not overwritten by later genesis input.
- Explicit redb integrity check.

Not yet covered: process-kill/power-loss fault injection, disk-full and injected I/O errors, cross-process races, migration interruption, backup/restore, long-running soak tests, benchmarks, and independent review.
