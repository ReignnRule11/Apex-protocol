# Phase 10 — redb Durable Storage Spike

**Status:** prototype implemented; GitHub Actions passed on Rust 1.90.0 (format, Clippy, workspace tests). Extended fault-injection and operational validation remain open.

## Objective

Implement a durable reference adapter behind the backend-neutral state-store interface while preserving atomic version/account updates and the canonical-codec boundary.

## Decisions

- redb 4.5.0 is pinned exactly in a separate `apex-state-redb` crate.
- Workspace MSRV and CI are pinned to Rust 1.90.0, matching redb's upstream manifest.
- redb default exclusive-file locking is used; no multi-process mode is enabled.
- Default `Durability::Immediate` is retained.
- Account records use a private 24-byte LE storage representation and explicit schema version.
- The StateStore snapshot method is fallible; backend errors cannot become panics or empty state.

## Implemented scope

- Persistent metadata and account tables.
- First-open initialization that does not overwrite an existing initialized database.
- Consistent read-transaction snapshots.
- One-transaction compare-and-swap block application.
- Before-state validation before the first write.
- Version overflow and stale-plan rejection.
- Explicit integrity-check method.
- Close/reopen and atomicity regression tests.
- CI validation on the merged PR head: format, Clippy and workspace tests passed.

## Not implemented

No canonical account encoding, protocol state root, Merkle proofs, block hash binding, transaction index, receipts, EVM state, pruning, state sync, multi-process mode, backup/restore workflow, process-kill/power-failure harness, disk-full injection, or production operational tooling.

## Acceptance criteria

- [x] Verify exact backend version, license, MSRV and durability semantics.
- [x] Keep backend isolated from protocol commitment logic.
- [x] Implement atomic account + version transaction.
- [x] Surface backend failures through fallible StateStore methods.
- [x] Test close/reopen persistence.
- [x] Test stale-plan rejection and late precondition failure.
- [x] Ensure existing data cannot be overwritten by bootstrap input.
- [x] Pass CI on the exact pinned toolchain.
- [ ] Add fault-injection/process interruption tests.
- [ ] Validate concurrent access and backup/restore.
- [ ] Complete dependency audit and independent storage review.

## Next gate

Do not treat the adapter as production storage until fault injection, multi-process semantics, backup/restore, schema migration, and independent review are complete. State commitments remain a separate protocol decision.
