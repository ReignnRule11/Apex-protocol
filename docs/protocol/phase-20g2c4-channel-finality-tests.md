# Phase 20G2C4 — deterministic Malachite channel finality tests

**Status:** Merged; post-merge main CI passed on workflow #42421014413. Production adoption remains blocked.
**Base main:** `27c076082a7e6035f8ff8ddabb2fb12829087a4e`
**Merge commit:** `7f2945c629f2eea41779f0f210038f7f8094a5c7`
**Pinned Malachite:** v0.8.2, `a453b32155f4ba9f959724279b52f601bae92e09`.

## Objective

Turn the Phase 20G2C3 finality-gate ordering contract into deterministic tests against the actual pinned Malachite channel message, certificate, engine response and Tokio receiver types. Tests demonstrate fail-closed acknowledgement behavior without substituting a fake consensus engine for upstream API types.

## Delivered

- Added an isolated dev-dependency on the exact-revision upstream `arc-malachitebft-test` fixture crate and deterministic RNG support. It remains inside `experiments/malachite-spike`; the production workspace dependency graph is unchanged.
- Extracted the receiver-driving loop so the same `mpsc::Receiver<AppMsg<Ctx>>` path used by `Channels::consensus` can be exercised without constructing unrelated network/event/request actors.
- Added test-only construction for the opaque verified-finality token, guarded by `cfg(test)`.
- Added a deterministic application fixture that records callback order and injects verification, staging, commit and restart-parameter failures.

## Test matrix

1. Decided: verify, stage, then acknowledge; no application commit and no next-height instruction.
2. Decided verification or staging failure: acknowledgement sender is dropped.
3. Finalized success: verify and commit callback precede `Next::Start(height + 1)`.
4. Finalized commit failure: `Next::Restart(current_height)` is returned.
5. Finalized verification failure or verified-identity height mismatch: no commit; restart is requested.
6. Restart-parameter failure: Finalized reply is dropped, leaving consensus unable to advance.
7. Height overflow: verification/commit/start are skipped; restart path is used.
8. Actual consensus receiver: a queued Decided message is processed and acknowledged before receiver closure is reported.

## Boundaries and limitations

- Application callbacks are deterministic test fixtures. They do not call the native Malachite certificate verifier or decode canonical APEX blocks.
- The verified-token test constructor cannot be called in non-test builds.
- Test validator keys and sets are synthetic, not production credentials or authenticated governance state.
- The receiver test exercises the exact Tokio receiver type and shared receive loop, but does not construct or start a live `EngineBuilder` runtime.
- No canonical APEX value store/codec, native/APEX validator key-power mapping, durable application database, consensus/application atomic commit, signer anti-rollback, crash-recovery proof, network fault test or BFT safety/liveness proof is delivered.
- CI success proves compilation and the stated test assertions only. G2 remains BLOCKED and the overall release remains NO-GO.

## Next

Implement bounded canonical APEX value retrieval and authenticated validator-set provenance behind a concrete isolated `FinalityApplication`, then run live multi-validator integration with durable commit/restart and fault injection. Do not promote the dependency into the production workspace until those gates and independent review pass.
