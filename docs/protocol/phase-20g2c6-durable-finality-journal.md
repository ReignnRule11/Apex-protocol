# Phase 20G2C6 — isolated durable finality journal prototype

**Status:** Implementation submitted for exact-head CI; not production-approved.  
**Base main:** `615e053c64a11a31edce3ea3c39a492437625b22`  
**Pinned Malachite:** v0.8.2, `a453b32155f4ba9f959724279b52f601bae92e09`.

## Objective

Turn the Phase 20G2C5 idempotent commit contract into a concrete, bounded persistence experiment without promoting Malachite or redb into the production workspace.

## Delivered in this phase

- Adds `finality_journal` to the isolated `experiments/malachite-spike` workspace and pins redb 4.5.0 for the experiment.
- Stores the complete APEX identity and opaque application-state payload in one journal record.
- Stores the chain tip in a second table updated in the same redb write transaction as the journal record.
- Derives the storage key from `FinalityCommitKey`, which is obtainable from the opaque verified-native-finality token.
- Enforces same-key/same-payload idempotency; rejects a conflicting identity or payload at the same height.
- Requires the first journaled finalized block to be height 1 with a zero parent-hash genesis anchor; subsequent records must be contiguous and parent-linked.
- Bounds payload size at construction and supports reopen/readback of persisted payloads.
- Treats a write-transaction commit error as ambiguous; it must not be translated into a same-height retry.

## Test intent

The isolated tests cover reopen/readback, exact retry, conflicting finality, payload conflict semantics, height continuity, parent linkage and payload bounds. Exact-head GitHub Actions remains the authoritative compile/test gate; no local Rust test is claimed.

## Boundaries and unresolved risks

- This is a single-process/single-writer redb prototype; no multi-process locking or concurrency guarantee is claimed.
- No OS crash/power-loss injection, fsync ordering audit, filesystem/device matrix, backup rollback protection, corruption repair or authenticated snapshot restore is proven.
- The payload is opaque bytes; canonical APEX block decoding, full deterministic application execution, complete state snapshot/delta semantics and schema migration are not implemented.
- The journal is not yet wired into `FinalityApplication::commit_finalized`; no live Malachite engine or validator is running.
- The verified token is still only as trustworthy as the upstream caller's canonical-byte/value-ID binding and validator-set provenance implementation.
- This does not provide atomicity across application state, consensus WAL and signer reservations.

**G2 and G4 remain BLOCKED. Overall release remains NO-GO.**

## Next

Exercise process termination and recovery at each write boundary, then define a separately reviewed atomic integration boundary spanning application state, consensus WAL and signer reservations. Do not enable validator keys.
