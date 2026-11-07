# Phase 20B — Isolated Malachite Dependency/API Spike

**Status:** Implementation and exact-head CI validation in progress.
**Base main:** `104cec631eca96ef4e95adcb355963cd8944a93b`
**Release impact:** G2 remains BLOCKED. Production remains NO-GO.

## Objective

Prove that APEX's Rust 1.91 toolchain can resolve and compile the pinned Malachite v0.8.2 application facade, expose the intended builder API, and produce inspectable dependency/license evidence without adding Malachite to the APEX production workspace.

## Upstream evidence

- Release: Malachite v0.8.2.
- Exact signed tag target: `a453b32155f4ba9f959724279b52f601bae92e09`.
- Tag object: `ed43c89a3a5711f4ae148a486a377a1bf4bdff38`; GitHub reports the annotated tag signature as verified.
- Facade package: `arc-malachitebft-app-channel`.
- Upstream manifest declares Apache-2.0 and Rust 1.88; APEX uses Rust 1.91.0.
- The facade depends on the engine, application, config, signing, network, sync and WAL crates. This is a material transitive dependency surface, not a lightweight library.

## Implementation

A standalone Cargo workspace is added under `experiments/malachite-spike`:

- `Cargo.toml` depends only on the upstream channel facade, with an exact Git revision and default features disabled.
- `src/lib.rs` re-exports the public EngineBuilder and actor context types and records the pinned release/revision.
- `tests/api_smoke.rs` checks the release pin and constructs the public RequestContext.
- The root APEX Cargo workspace remains unchanged; the experiment is not a production member.

The existing Rust CI workflow gains a separate job that generates the isolated lockfile, runs `cargo check --all-targets`, runs the smoke tests, captures `cargo tree` and `cargo metadata`, and uploads the lockfile/dependency/license report as a 30-day artifact.

## Safety boundaries

This is an API compilation probe only. It does not instantiate EngineBuilder, create a validator signer, create a network identity, bind a socket, start a WAL, spawn consensus actors, produce proposals or establish finality. It does not validate an APEX application callback or commit boundary.

The generated lockfile is a CI artifact, not yet a committed lockfile. Review the exact resolved dependency graph and then commit a reviewed lockfile before treating the dependency resolution as reproducible. Package license metadata is an inventory, not legal clearance or a complete license compatibility opinion. The workflow does not yet perform a RustSec advisory audit.

The full upstream workspace test suite has not yet been reproduced. The two local smoke tests do not substitute for it.

## Exit criteria

1. Exact-head CI compiles the standalone crate and passes its tests.
2. The lockfile artifact is captured and inspected; exact crate versions and Git source are confirmed.
3. Dependency tree and license metadata are reviewed for unexpected packages, undeclared licenses and policy conflicts.
4. Upstream tests are run at the exact pinned revision, with test scope and failures recorded.
5. The application, engine, network, signer, WAL and shutdown interfaces are mapped before a live-node experiment.

## Next work

If the isolated build and dependency review pass, Phase 20C will define an engine-neutral APEX application boundary. Do not instantiate a validator or network in this phase.

**Decision: ISOLATED COMPILE PROBE GO; PRODUCTION CONSENSUS ADOPTION NO-GO.**
