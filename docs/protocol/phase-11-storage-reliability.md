# Phase 11 — Storage Reliability and Recovery Validation

**Status:** reliability test expansion implemented in the Phase 11 branch; CI and review pending. This phase does not certify production storage.

## Objective

Exercise the redb adapter at its recovery boundary and document which failure classes are tested versus still untested. Preserve the invariant that account updates and StateVersion are one atomic commit.

## Implemented validation

- **Uncommitted process interruption:** a child test process opens a write transaction, changes an account record without committing, and aborts. The parent reopens the database and verifies that the previous committed StateVersion and account value remain.
- **Committed process interruption:** a child process applies a versioned block plan, receives a successful return from the immediate-durability commit, then aborts. The parent reopens and verifies the new StateVersion and all expected account values.
- **Snapshot stability:** a materialized snapshot remains unchanged after a later commit; a fresh snapshot observes the new version.
- **Cold backup/restore:** after the database handle is closed, the test copies the file and verifies that the restored database produces the same snapshot.
- **Schema rejection:** an unknown metadata schema version is rejected rather than interpreted as the current layout.

The child-process tests use abrupt process termination. They do not reproduce an OS crash, power loss, torn sector, volatile device cache loss, filesystem bug, or controller failure.

## Recovery and error semantics

1. redb performs its documented recovery when a database is opened after an unclean shutdown.
2. APEX must not automatically retry a failed commit. A commit I/O error can leave the caller unable to infer whether the transaction became durable. Close/reopen and reconcile the persisted StateVersion and affected records before deciding what to do.
3. A failed commit must not be treated as proof that no state was applied.
4. Unknown schema versions and malformed metadata/accounts remain fail-closed.
5. check_integrity() is an explicit diagnostic/repair operation, not a per-request health check. It requires no active read/write transaction on that database handle.
6. A cold backup is a file copy taken only after a successful close. No live-copy or online-backup guarantee is exposed.

## Test boundaries and outstanding work

| Area | Phase 11 evidence | Remaining gap |
|---|---|---|
| Uncommitted write | Child process abort + reopen | OS/power-loss testing |
| Successful commit | Abort immediately after commit returns + reopen | Hardware durability certification |
| Snapshot consistency | Old and new materialized snapshots compared | Sustained concurrent load |
| Backup/restore | Closed-file copy + reopen + state comparison | Automated rotation, checksums, off-site restore drills |
| Schema handling | Unknown version rejected | Versioned migration framework and interrupted migration tests |
| I/O failure | Not simulated | Disk-full, short-write, permission and device error injection |
| Concurrency | Existing redb MVCC contract only | APEX concurrent readers/writers and multi-process modes |

## Acceptance criteria

- [x] Demonstrate process-abort recovery before commit.
- [x] Demonstrate reopen after a successful commit followed by process abort.
- [x] Verify materialized snapshot immutability across later writes.
- [x] Verify a cold backup can be reopened with equivalent state.
- [x] Verify unsupported schema versions fail closed.
- [ ] Add deterministic storage-backend I/O fault injection.
- [ ] Test disk-full and interrupted migration behavior.
- [ ] Test APEX-level concurrent snapshots and serialized writers.
- [ ] Perform repeatable restore drills and long-running soak tests.
- [ ] Obtain independent storage/security review.

## Next gate

Do not mark the adapter production-ready. The next storage gate is controlled I/O failure injection, explicit recovery/reconciliation APIs, schema migration design, and concurrency testing. State commitments remain blocked on canonical state encoding.
