# Phase 20G2C7 — finality journal process-termination recovery tests

**Status:** Implementation submitted for exact-head CI; isolated research only.  
**Base main:** `64bdeffe9e7bd3a225269d41f70cb139e2217ead`  
**Preceding phase:** 20G2C6, merged with post-merge main CI passing.  
**Pinned Malachite:** v0.8.2, `a453b32155f4ba9f959724279b52f601bae92e09`.

## Objective

Test whether the isolated redb finality journal reopens in a consistent, fail-closed state after abrupt process termination at two decisive transaction boundaries: after both journal/tip rows are staged but before commit, and after commit returns but before the caller receives its acknowledgement.

## Delivered in this phase

- Refactors the private journal commit path to expose internal-only before-commit and after-commit test hooks; the public API remains unchanged.
- Adds a child-process test probe that calls `std::process::exit`, bypassing Rust destructors and simulating abrupt process termination.
- Before-commit termination test verifies that reopening exposes neither a journal payload nor a chain tip, and that the same-height commit can then be safely performed.
- After-commit termination test verifies that reopening exposes the complete payload and tip, and that an exact same-key/same-payload retry returns `AlreadyCommitted`.
- Exercises recovery through a fresh process and a newly opened database handle rather than relying on the original process's in-memory state.

## Test interpretation

These tests exercise process termination on the CI host at controlled application transaction boundaries. They are not physical power-cut tests, kernel crash tests, storage-controller fault injection, filesystem durability certification, fsync-ordering audits, or proof that every operating system/device honors the same persistence guarantees. The before-commit and after-commit boundaries are explicit test hooks, not instrumentation of redb internals.

## Boundaries and unresolved risks

- The journal remains a single-process/single-writer prototype; concurrent writers and multi-process coordination are not tested.
- No power-loss, kernel panic, torn-write, disk-full, I/O-error, filesystem/device matrix, backup rollback, corruption repair, authenticated restore or schema migration evidence is provided.
- The state payload remains opaque; canonical APEX block decoding and deterministic application-state reconstruction are not implemented.
- The journal remains disconnected from `FinalityApplication::commit_finalized`, Malachite's WAL, and the signer reservation journal. No cross-subsystem atomicity is established.
- No live engine, validator, network, BFT safety/liveness proof or production key operation is included.

**G2 and G4 remain BLOCKED. Overall release remains NO-GO.**

## Next

Design and review an integration protocol that reconciles application state, Malachite consensus WAL and signer reservations across restart without assuming a distributed atomic transaction. Specify recovery authority, commit markers, replay rules, rollback detection and fail-closed behavior before implementing any live engine integration. Do not enable validator keys.
