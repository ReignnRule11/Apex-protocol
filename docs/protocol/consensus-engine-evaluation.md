# Consensus Engine Evaluation — Phase 2 and Phase 20A Revalidation

**Decision status:** Malachite v0.8.2 remains the preferred Rust-native candidate for an isolated research integration spike only. Production adoption is NOT approved. CometBFT remains the reference/fallback comparator.

## APEX requirements

APEX requires BFT proof-of-stake consensus with deterministic state-machine replication, explicit safety/liveness assumptions, validator-set changes, authenticated votes, verifiable commit certificates, crash recovery, bounded resource use, testable networking behavior and a maintainable upgrade path. Rust is the selected implementation language.

## Candidate: Malachite

Repository: https://github.com/circlefin/malachite

- Rust-native Tendermint-style BFT consensus engine.
- Release: **v0.8.2**, published 2026-10-19.
- Exact upstream source commit: **a453b32155f4ba9f959724279b52f601bae92e09**.
- Upstream workspace version: **0.8.2**; declared MSRV: **Rust 1.88**.
- APEX workspace MSRV: **Rust 1.91.0**; compatible. The earlier MSRV mismatch is resolved.
- Upstream license: **Apache-2.0**.
- Upstream README explicitly labels Malachite alpha, under heavy development and not externally audited.
- Upstream describes Malachite as the consensus foundation for Arc. This is evidence of active engineering/use, not independent assurance or proof of APEX suitability.

### Release-specific evidence

The v0.8.2 release notes include fixes involving proposal round validation, commit-certificate round handling during sync, and validator-proof session behavior. The upstream release commit reports a five-validator rolling upgrade from 0.8.1 to 0.8.2 under load. These are upstream signals, not APEX test results or independent assurance.

### Integration risks

- Alpha maturity and API/release churn.
- No external audit statement from upstream.
- Multi-crate dependency graph spanning consensus, engine, application, networking, sync, peer, WAL, signing, metrics and codecs.
- APEX-specific signer safety, durable anti-double-sign, application atomicity, validator transitions and recovery are not inherited by adding the engine.
- Exact feature selection and transitive dependency review remain outstanding.
- Engine signing and wire-format assumptions must not silently become APEX protocol decisions.

### Phase 20A decision

Pin **v0.8.2 / a453b32155f4ba9f959724279b52f601bae92e09** as the research baseline for an isolated spike. Do not add the dependency to the production workspace, connect validator keys, or claim APEX has BFT finality.

The Rust 1.91.0 toolchain satisfies the upstream 1.88 MSRV. This removes the toolchain incompatibility but not maturity, dependency, API, signer or assurance gates.

## CometBFT comparator

Repository: https://github.com/cometbft/cometbft

- Go implementation and successor to Tendermint Core.
- Well-documented consensus and ABCI/ABCI++ application boundary.
- Credible reference/fallback if Malachite API stability, assurance or operational fit fails.
- Costs include a Go runtime/process boundary or interoperability layer in a Rust-first node.

## Phase 20B isolated API spike

The isolated experiment is located at `experiments/malachite-spike`. It depends only on the public channel-based application crate `arc-malachitebft-app-channel`, pinned by Git revision to the v0.8.2 tag commit. The crate transitively brings in Malachite's application, engine, network, sync, signing and WAL crates. The experiment re-exports selected public builder/context types and smoke-tests the request-context API. This is a compile/API compatibility probe, not a running consensus node. The experiment has its own Cargo workspace and does not modify the APEX root workspace dependency graph. CI generates a lockfile and publishes the lockfile, dependency tree and license metadata as an artifact for inspection.

The isolated dependency is deliberately limited to the channel application facade: this tests the public integration entry point and makes its transitive scope visible. No default validator signer is instantiated; no network identity, listening socket, WAL path, peer, consensus actor or block is created. The isolated compile and smoke tests passed on the exact APEX PR head. The generated dependency evidence contains 415 package records (397 registry, 17 exact-revision Malachite Git packages, one local path); all declare license metadata, but no legal compatibility conclusion is drawn. The upstream Rust CI unit-test command is now reproduced in a separate APEX CI job using cargo-nextest and the exact source revision. Upstream Maelstrom integration tests and Quint/MBT model checking remain out of scope for this first spike.

## Ordered Phase 20 gates

1. **20A — candidate revalidation:** exact upstream version/commit, license, MSRV and maturity recorded. Complete.
2. **20B — isolated dependency/API spike:** exact crate selection, Cargo.lock resolution, full dependency/license/advisory review, reproducible upstream tests, and a minimal isolated engine build.
3. **20C — application boundary:** map proposal validation, deterministic application callbacks, finalized decision verification, replay and commit handoff.
4. **20D — signer safety:** ratify a dedicated consensus signature suite and domain; test durable anti-double-sign across crash, restore/backup and key rotation.
5. **20E — validator/finality semantics:** validator-set updates, activation/exit boundaries, certificate verification and equivocation evidence.
6. **20F — adversarial integration:** at least four independent processes; Byzantine behavior, conflicting proposals, partitions, delayed/dropped/reordered messages, restart/recovery, churn, deterministic replay, resource exhaustion and CI reproduction.
7. **Independent assurance:** review pinned code/dependency graph, protocol assumptions, signer lifecycle and test evidence before public testnet.

## Explicit non-decisions

No consensus signature suite, numeric chain ID, canonical consensus message codec, validator economics, block interval, finality target, public validator policy or mainnet launch date is approved. No Malachite dependency is currently present in APEX Cargo manifests.

## Sources reviewed

- Malachite v0.8.2 release: https://github.com/circlefin/malachite/releases/tag/v0.8.2
- Exact upstream commit: https://github.com/circlefin/malachite/commit/a453b32155f4ba9f959724279b52f601bae92e09
- Upstream workspace manifest: https://github.com/circlefin/malachite/blob/v0.8.2/code/Cargo.toml
- Upstream license: https://github.com/circlefin/malachite/blob/v0.8.2/LICENSE
- Upstream release notes: https://github.com/circlefin/malachite/blob/v0.8.2/RELEASE_NOTES.md
- CometBFT repository: https://github.com/cometbft/cometbft
