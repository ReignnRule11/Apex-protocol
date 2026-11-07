# ADR-0040: Isolated Malachite v0.8.2 API Spike

**Status:** Accepted for isolated research only; production adoption NOT approved.
**Date:** 2026-11-07

## Context

Phase 20A revalidated Malachite v0.8.2 as the preferred Rust-native candidate for a contained integration study. Its Rust 1.88 MSRV is compatible with APEX Rust 1.91.0, but upstream explicitly labels it alpha and not externally audited. The application facade transitively includes consensus engine, networking, sync, signing, WAL and supporting crates.

## Decision

Create `experiments/malachite-spike` as a separate Cargo workspace. Depend only on `arc-malachitebft-app-channel` at exact Git revision `a453b32155f4ba9f959724279b52f601bae92e09`, with default features disabled. Re-export the public builder/context types and compile a minimal API smoke test.

Keep this workspace outside the APEX root workspace. Do not add Malachite to production Cargo manifests, connect validator keys, spawn consensus/network actors, or claim finality.

## Dependency evidence

- Upstream package manifest identifies the facade package as `arc-malachitebft-app-channel`.
- Its dependencies include `arc-malachitebft-app`, `arc-malachitebft-engine`, config, signing and the network/sync/WAL stack.
- The facade's optional `byzantine` feature is not enabled.
- APEX CI generates an isolated Cargo.lock and uploads it with `cargo tree`, Cargo metadata and a package-license inventory.
- The lockfile is an artifact at this stage, not a committed source lock. A reviewed committed lockfile is a remaining reproducibility gate.
- License metadata inventory does not replace legal review; RustSec advisory scanning remains outstanding.

## Consequences

- The public integration facade can be evaluated without contaminating APEX production dependencies.
- The experiment exposes the true transitive dependency scope for review.
- A successful compile proves API/toolchain compatibility only; it does not prove consensus safety, application correctness, signer safety, networking security or operational suitability.
- If dependency size, license policy, advisory status, API churn or operational coupling is unacceptable, stop and reassess CometBFT or another candidate.

## References

- Malachite v0.8.2 signed tag: https://github.com/circlefin/malachite/releases/tag/v0.8.2
- Exact tag target: https://github.com/circlefin/malachite/commit/a453b32155f4ba9f959724279b52f601bae92e09
- App-channel manifest: https://github.com/circlefin/malachite/blob/v0.8.2/code/crates/app-channel/Cargo.toml
- App-channel builder API: https://github.com/circlefin/malachite/blob/v0.8.2/code/crates/app-channel/src/builder.rs
- Upstream workspace manifest: https://github.com/circlefin/malachite/blob/v0.8.2/code/Cargo.toml
- Phase 20A record: docs/protocol/phase-20a-consensus-engine-revalidation.md
