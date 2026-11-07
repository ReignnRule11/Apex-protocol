# Phase 12 — Storage Concurrency and Failure-Boundary Engineering

**Status:** implementation and tests added; CI/review pending. Production approval withheld.

## Objectives

Test the concurrency contract exposed by the current single-process redb adapter and make ambiguous commit acknowledgement behavior executable. Keep simulated failures clearly separated from real filesystem/device fault injection.

## Implemented

### Concurrent readers
- Eight threads share an `Arc<RedbStateStore>` and repeatedly request snapshots.
- Each snapshot is checked for one expected StateVersion and a complete set of account records.
- This tests concurrent read access through one database handle; it does not test a reader racing a writer in the current `&mut self` adapter API.

### Serialized competing writers
- Two plans are produced from the same parent snapshot.
- Two threads submit those plans through a process-local `Mutex<RedbStateStore>`.
- Exactly one applies; the other receives `StaleVersion` after observing the incremented StateVersion.
- This validates serialized caller access and optimistic version rejection, not independent processes or redb multi-process writer mode.

### Fault-boundary injection
- **Before apply:** a test decorator returns a backend error before calling redb; the snapshot remains unchanged.
- **After successful apply:** the decorator allows the redb adapter to commit and then simulates loss of the acknowledgement by returning a backend error.
- The test drops the adapter, reopens the database, checks the new version and account values, then retries the original plan and verifies stale-plan rejection.

The after-commit test models an ambiguous acknowledgement at the `StateStore` boundary. It does not induce a redb commit error, disk-full condition, filesystem failure, short write, permission error or device fault.

## Operational rule: reconcile before retry

If `apply` returns an error whose commit outcome may be ambiguous:
1. Do not blindly retry the same plan.
2. Stop accepting further writes through that adapter instance until it has been closed/reopened or otherwise reconciled.
3. Read the persisted StateVersion and relevant state from a fresh snapshot.
4. If the plan's effects and expected version are present, treat it as committed; if the parent version remains current, a controlled retry may be considered; if state is inconsistent or unreadable, fail closed and escalate.
5. Any future implementation must use a durable block/operation identity and deterministic replay policy before enabling automated retries.

This phase tests the reconciliation principle using a lost-acknowledgement simulation. It does not yet add a production recovery state machine or operation journal.

## Explicit boundaries

| Area | Evidence | Not established |
|---|---|---|
| Concurrent snapshots | Multiple reader threads, one shared handle | Concurrent read/write load or multi-process reads |
| Competing writers | Mutex-serialized submissions; one succeeds, one stale | Independent processes, lock contention under production load |
| Pre-apply failure | Deterministic wrapper returns before delegation | redb/filesystem failure atomicity |
| Ambiguous acknowledgement | Commit succeeds, wrapper loses acknowledgement, reopen reconciles | Actual commit I/O error semantics |
| Disk capacity and I/O | Not tested | Disk-full, ENOSPC, EIO, permission, short-write, media faults |
| Schema migration | Design remains deferred | Migration, rollback, crash interruption |

## Acceptance criteria

- [x] Concurrent snapshot readers observe complete snapshots at one version.
- [x] Competing plans for the same parent version produce one successful apply and one stale rejection when serialized.
- [x] Pre-apply injected failure leaves persisted state unchanged.
- [x] Lost acknowledgement after successful commit is reconciled by reopen; replay is rejected.
- [ ] Exercise reader/writer overlap through a deliberately designed concurrent adapter API.
- [ ] Test independent-process locking and writer contention.
- [ ] Add real filesystem/storage I/O fault injection and disk-full tests.
- [ ] Design and test schema migration with rollback and interruption.
- [ ] Add soak tests, performance baselines, and independent review.

## Next gate

Do not enable multi-process redb mode or call the adapter production-ready. Before that, design the adapter's concurrent API and lifecycle, test supported process-locking semantics, add real I/O fault injection, and specify an explicit schema migration/recovery state machine. Canonical state commitments remain a separate blocked decision.
