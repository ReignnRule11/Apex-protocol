# Persistent State Storage Architecture — Phases 10–12

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

A snapshot is read inside one redb read transaction and materialized into a sorted BTreeMap. The snapshot contains one committed StateVersion. The adapter uses redb's default exclusive-file locking mode and remains single-process/single-handle. Phase 12 exercises concurrent read transactions through a shared adapter reference and serialized writers through a process-local mutex. Competing plans built from the same parent version produce one success and one stale-version rejection. This does not enable or test multiple independent processes or redb's optional multi-process concurrency mode.

## Failure injection and ambiguous commit handling

Phase 12 adds a test-only `StateStore` decorator with two controlled failure points:
- Before delegation: returns an injected backend error without invoking the redb adapter; the persisted snapshot remains unchanged.
- After delegated apply returns success: returns an injected lost-acknowledgement error even though the underlying commit has completed. The test closes and reopens the database, observes the incremented StateVersion and account values, and confirms that retrying the original plan is rejected as stale.

This validates APEX caller behavior when an acknowledgement is lost. It does **not** inject an actual redb, filesystem, disk-full, short-write, permission, or device I/O error. It cannot prove backend failure atomicity for those conditions.

## Recovery and operational requirements

- redb performs its documented recovery when the database is opened.
- The adapter exposes an explicit integrity-check method.
- Malformed metadata, unsupported schema versions, and malformed account records return errors; no defaults are synthesized.
- The tested backup procedure is a cold file copy only: close the database successfully, then copy the file, then reopen the copy and compare the recovered snapshot. Copying a live database file is not supported. No online backup API is exposed by APEX.
- Immediate durability is the redb default and is retained; the adapter does not request non-durable commits.
- No claim is made about power-loss behavior beyond the backend's documented contract and supported filesystem/platform assumptions.

## redb version and compatibility

The adapter pins redb 4.5.0 exactly. Its upstream Cargo manifest declares Rust 1.90 and MIT OR Apache-2.0 licensing. The 4.5.0 release is recent; its release notes describe multi-process concurrency modes and fixes to integrity checking and page-cache behavior. The workspace MSRV and CI toolchain are therefore explicitly raised to 1.90.0 for this contained evaluation.

## Conformance status

Implemented tests cover:
- Bootstrap and first initialization.
- Durable multi-account application and version increment.
- Close/reopen persistence and stable materialized snapshots.
- Stale-plan rejection and late before-state mismatch with no partial writes.
- Existing database is not overwritten by later genesis input.
- Explicit redb integrity check.
- Abrupt subprocess termination with an uncommitted write; reopen preserves the prior committed version and account values.
- Abrupt subprocess termination immediately after a successful commit; reopen observes the committed version and account values.
- Cold backup by copying a closed database file and restoring it with equivalent snapshot contents.
- Unknown schema versions fail closed.

Still not covered: true power interruption, OS/kernel crash, disk-full and actual storage I/O errors, cross-process races, independent multi-process writers, migration interruption/rollback, long-running soak tests, benchmarks, and independent review. Phase 12's concurrent-reader test and mutex-serialized competing-writer test are process-local. Its injected failures simulate adapter-boundary outcomes, not physical I/O failures. The subprocess tests exercise process termination only; they do not simulate loss of kernel page cache, storage-controller cache, or physical power.
