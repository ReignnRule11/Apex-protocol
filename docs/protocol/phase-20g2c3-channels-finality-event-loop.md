# Phase 20G2C3 — Malachite Channels finality event-loop gate

**Status:** Merged; deterministic channel-level behavior tests are tracked in Phase 20G2C4. Production adoption remains blocked.
**Base main:** `0b8800718db65343420808366f782a851e7adefb`
**Pinned Malachite:** v0.8.2, `a453b32155f4ba9f959724279b52f601bae92e09`.

## Delivered

- Added `finality_event_loop.rs` to the isolated `experiments/malachite-spike` workspace.
- The gate consumes the actual `AppMsg` types emitted by the pinned `Channels::consensus` receiver.
- Added `run_channels_finality_gate`, which receives from the real channel and delegates all non-finality messages unchanged to the application's ordinary handler.
- Added a `FinalityApplication` contract separating native certificate verification/value-ID binding, early decision staging, finalized durable commit and restart-parameter retrieval.
- `Decided` is verified and staged before its acknowledgement is sent. It never emits `Next::Start` or advances the application height.
- `Finalized` is reverified against its final certificate, then committed. The gate emits `Next::Start(height + 1, params)` only after the commit callback succeeds and the next-height increment is checked.
- Verification, identity mismatch, height overflow or commit failure takes a fail-closed `Next::Restart(current_height, params)` path. If restart parameters cannot be obtained, the reply is dropped and consensus remains stalled rather than being advanced.
- Pinned the exact-revision Malachite engine crate in the isolated workspace so the gate uses the upstream `Next` type. No production workspace dependency was added.

## Required application implementation

The `FinalityApplication` implementation must:
1. Retrieve the value by native value ID under strict byte/count limits.
2. Strictly decode canonical APEX block bytes and recompute block hash and parent linkage.
3. Select an authenticated native validator set and explicit threshold parameters for the exact height.
4. Call `verify_and_bind_native_certificate`, which invokes Malachite's native `VerifierExt`.
5. Stage a `Decided` value in non-finalized storage only.
6. On `Finalized`, reverify the possibly extended certificate and atomically persist finalized application state before returning next-height parameters.

The event-loop gate enforces call ordering but cannot prove that a user-supplied implementation performs those operations correctly. The ordinary handler for every other `AppMsg` remains application-owned.

## Validation and limitations

- At initial submission, the only unit test checked u64 height overflow. Phase 20G2C4 adds deterministic callback-ordering and actual receiver tests; those tests remain isolated fixtures and do not execute a live engine or real cryptographic verification.
- Exact-head CI must compile the generic gate against the pinned Malachite channel and engine APIs and run the isolated workspace checks.
- No live Malachite engine has been started; no real validators, keys, sockets or production database are used.
- No concrete APEX `FinalityApplication` implementation, canonical value codec, authenticated validator-governance mapping, durable cross-subsystem atomic commit, signer anti-rollback, crash recovery or BFT safety/liveness proof is delivered.
- This is an event-loop gate primitive, not an integrated production consensus runtime. G2 remains BLOCKED and overall release remains NO-GO.

## Follow-on work

20G2C4 adds deterministic tests. Remaining implementation work is a concrete isolated adapter using bounded canonical APEX value retrieval and authenticated validator-set provenance. The required test coverage includes: `Decided` staging without height advancement; invalid native certificates; value-ID binding failure; APEX chain/version/height/parent mismatch; successful `Finalized` commit followed by exactly one next-height start; commit failure followed by restart; restart-parameter failure; and channel closure. Then build a concrete isolated adapter using bounded canonical APEX value retrieval and authenticated validator-set provenance.
