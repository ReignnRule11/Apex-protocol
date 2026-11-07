# ADR-0015: Concurrency and Ambiguous Commit Handling

- **Status:** Accepted for prototype test semantics; production recovery design incomplete
- **Date:** 2026-11-07

## Context

The `StateStore` API currently exposes `snapshot(&self)` and `apply(&mut self)`. redb supports concurrent read transactions, but the mutable apply signature does not permit a caller to share one adapter instance for overlapping snapshot and apply operations without an external synchronization design. Concurrent writers in the current prototype must be serialized. Commit errors can also create an ambiguous outcome for callers.

## Decisions

1. Test multiple concurrent snapshot readers against one shared adapter handle.
2. Test competing plans from one parent StateVersion through a process-local mutex; exactly one plan may apply.
3. Add test-only failure injection before delegation and after a successful delegated commit.
4. Treat the post-commit injected error as lost acknowledgement, not as a real backend I/O error.
5. Require reopen/reconciliation of persisted StateVersion and state before retrying an operation whose commit result is ambiguous.
6. Do not enable redb multi-process concurrency modes in this phase.

## Consequences

The tests establish process-local concurrent-reader behavior and optimistic stale-plan rejection under serialized writers. They do not establish reader/writer overlap through the APEX API, independent process locking, multi-process write safety, real I/O fault behavior, or production recovery. A future concurrent adapter may require an interior-mutability/transaction API redesign and an explicit lifecycle state that prevents writes while commit outcomes are unresolved.

## References

- APEX storage architecture: `docs/protocol/storage-architecture.md`
- redb 4.5.0 API: https://docs.rs/redb/4.5.0/redb/
