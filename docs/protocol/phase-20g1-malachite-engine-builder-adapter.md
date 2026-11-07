# Phase 20G1 — Pinned Malachite EngineBuilder Adapter

**Status:** Merged; post-merge main CI passed (run #42410559233).
**Base main:** `ab72067031e6c6a6cd4203d9f5acb7329b3183ad`
**Upstream:** Malachite v0.8.2, `a453b32155f4ba9f959724279b52f601bae92e09`.
**Release impact:** G2 and G3 remain BLOCKED. Production remains NO-GO.

## Objective

Move beyond a symbol-level API probe by compiling a typed adapter that calls the actual upstream `EngineBuilder::build` path. Keep the dependency isolated until runtime, signer, application and recovery contracts are independently demonstrated.

## Implementation

- Extend `experiments/malachite-spike`, the existing separate Cargo workspace.
- Add `build_engine`, a generic adapter over the upstream consensus context, node configuration and WAL/network/sync codecs.
- Configure all five required actor categories through the real builder: WAL, network, consensus, sync and request channels.
- Invoke the upstream asynchronous `build().await` method and return upstream `Channels` and `EngineHandle`.
- Keep the exact upstream git revision pinned; keep Malachite outside the root APEX workspace.
- CI compile-checks the generic adapter and runs the isolated API smoke tests.

## What this establishes

The public builder surface and its trait bounds are usable from an APEX-owned adapter crate. The adapter does not invent a parallel consensus engine or replace Malachite with a local simulation.

## Explicit limits

This phase does not call `build_engine` with a concrete node configuration and does not start Malachite actors. It creates no key material, network identity, listening socket, WAL file or validator process. It does not implement the APEX application event loop, canonical proposal/value codec, authenticated validator-set mapping, APEX signer adapter, anti-double-sign reservation enforcement, certificate verification, or atomic finalized-state commit. The adapter therefore does not demonstrate consensus progress, BFT safety/liveness, network recovery or APEX finality.

No consensus signature suite, chain ID, message domain, validator policy or network parameters are ratified by this phase. The dependency remains isolated and research-only.

## Exit criteria

1. Adapter calls the exact pinned upstream `EngineBuilder::build` API.
2. Generic bounds explicitly cover upstream context, config and all required codec roles.
3. CI compiles all targets with warnings denied in the isolated workspace.
4. Existing APEX workspace checks, process harness and pinned upstream unit suite remain green.
5. Documentation and readiness register state the runtime and security gaps without overstating completion.

## Next subphase

**20G2 — real isolated multi-validator runtime.** Implement a concrete APEX application over the upstream channel API, bridge APEX signer reservations to Malachite signing only after domain/codec decisions are reviewed, and run multiple independent validator processes with honest progress, invalid signatures, equivocation, partitions, restarts and commit-gating tests. Do not activate production keys or move the dependency into the root workspace.
