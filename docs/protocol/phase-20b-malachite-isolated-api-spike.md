# Phase 20B — Isolated Malachite Dependency/API Spike

**Status:** Complete; PR #38 merged and independently verified.
**PR:** https://github.com/ReignnRule11/Apex-protocol/pull/38
**Merge commit / main SHA at merge:** `7b52c164789905e16cd40b967e2fb4f905d3feb9`
**Exact-head CI:** https://github.com/ReignnRule11/Apex-protocol/actions/runs/42400799248 — success.
**Post-merge main CI:** https://github.com/ReignnRule11/Apex-protocol/actions/runs/42401375813 — success.
**Release impact:** G2 remains BLOCKED. Production remains NO-GO.

## Objective and implementation

A separate Cargo workspace at `experiments/malachite-spike` depends only on `arc-malachitebft-app-channel`, pinned to Malachite v0.8.2 commit `a453b32155f4ba9f959724279b52f601bae92e09`, with default features disabled. It re-exports selected builder/context API types and smoke-tests the public request context API.

The experiment remains outside the APEX root Cargo workspace and production dependency graph. It does not instantiate an engine, signer, network identity, socket, WAL, consensus actor or block.

## CI and dependency evidence

The exact-head workflow passed:
- APEX Format, Clippy and workspace tests.
- Isolated Malachite facade compile and API smoke tests.
- Pinned upstream v0.8.2 Rust unit-test command using cargo-nextest at the exact source revision.

The generated CI artifact contains 415 package records: 397 registry packages, 17 Malachite Git packages at the exact revision and one path package. There are 393 unique package names. All package records declare a license expression or license file. This is metadata completeness, not a legal compatibility opinion.

Artifact name: `malachite-v0.8.2-dependency-review`; artifact ID `13585282823`; SHA-256 `25ecffa3109db99326a926fca153510a25fb88731cdd3fa3cea263438dda048d`; retention through 2026-12-07.

The upstream Maelstrom integration tests and Quint/MBT model checking are not reproduced. RustSec advisory scanning and legal compatibility review remain outstanding. The generated lockfile is a CI artifact, not a committed source lock; reproducible dependency pinning remains a gate.

## Decision

**Isolated compile/API probe: PASS. Production consensus adoption: NOT APPROVED.**

No validator signer, network, consensus actor, proposal, finality or APEX BFT guarantee is enabled. Phase 20C defines the APEX-owned application boundary independently of the engine implementation.
