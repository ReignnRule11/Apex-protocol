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

## Phase 20C application boundary (merged)

PR #39 merged as `e7fd62b4646e3b022113f452175c1e00df2ad5ce`. The exact-head PR workflow and post-merge `main` workflow passed. The engine-neutral boundary checks proposal linkage, delegates deterministic validation, requires exact-identity finality verification and gates application commit. It remains an orchestration interface, not a production verifier or atomic commit protocol.

## Phase 20D durable signer safety (merged)

The new `apex-signer-safety` and `apex-signer-redb` crates persist exact-message reservations under a complete signing coordinate. A first reservation is committed before success is returned; an exact retry is idempotent; conflicting bytes are rejected. No signature algorithm, canonical consensus codec, key format or engine signing adapter is selected. Backup rollback and multi-process signing remain blockers. See the Phase 20D record and ADR-0042.

## Phase 20D durable signer safety (merged)

PR #40 is merged at `8a672599ea2a96abe2404b58afc997c2a6e42544`; post-merge `main` CI run #42404106956 passed all three jobs (workspace checks, isolated Malachite API spike and pinned upstream unit suite). The durable reservation prototype and its limitations are recorded in the Phase 20D document and ADR-0042.

## Phase 20E validator-set lifecycle primitives (merged; PR #41 and post-merge CI passed)

The new `apex-consensus::validator_set` module sorts validator entries by fixed-width opaque identity, rejects empty sets, all-zero identities, zero voting power and duplicate identities, and uses checked `u64` total-power arithmetic. Quorum accounting rejects duplicate/unknown supplied signer identities and requires strictly more than two-thirds of total power. A schedule applies each replacement set at its exact declared activation height and rejects zero/non-increasing activation heights. `EquivocationClaim` is only a typed envelope requiring two distinct caller-supplied digests; it does not prove either message was signed or authenticate the evidence.

No canonical validator-set hash/codec, authenticated governance/update path, signature algorithm, commit-certificate verifier, slashing policy or production finality is implemented. Activation schedule data is a local deterministic model, not a protocol ratification.

## Phase 20F multi-process adversarial harness (baseline implementation submitted)

The isolated workspace at `experiments/consensus-harness` launches four independent OS worker processes. Each process owns a separate append-only journal and exposes a bounded line command interface. The integration tests inject deterministic delivery schedules: a partitioned worker misses messages; reordered and delayed records are rejected until gaps are filled; dropped delivery is repaired on replay; conflicting content at an accepted sequence is rejected; a process restart reloads its journal; a replacement process catches up by replay; and repeated runs produce byte-identical journals. The worker rejects oversized payloads and caps journal record count.

This is intentionally a process/fault-injection fixture for the harness protocol, not a consensus implementation. It does not run Malachite, authenticate votes, sign messages, validate commit certificates, implement quorum agreement, use network sockets, model Byzantine message forgery, or establish BFT safety/liveness. A process-local journal is not a production WAL, and test replay is not authenticated state sync. CI runs isolated formatting, Clippy and integration tests. See `docs/protocol/phase-20f-multiprocess-adversarial-harness.md` and ADR-0044.

## Ordered Phase 20 gates

1. **20A — candidate revalidation:** exact upstream version/commit, license, MSRV and maturity recorded. Complete.
2. **20B — isolated dependency/API spike:** complete for the pinned compile/API and upstream Rust unit-test scope. Legal compatibility, RustSec review, committed lockfile and upstream Maelstrom/Quint suites remain open.
3. **20C — application boundary:** complete; PR #39 merged and post-merge CI passed. `apex-consensus::application_boundary` separates candidate validation, engine-specific finality verification and application commit. Its verifier is an interface contract only; no production verifier or durable replay/atomic commit protocol exists.
4. **20D — signer safety:** merged durable reservation prototype; signature suite/domain ratification, backup rollback protection, key custody and key rotation remain open.
5. **20E — validator-set lifecycle primitives:** deterministic set validation, checked voting power, quorum arithmetic, exact activation boundaries and typed equivocation claims. Authentication and certificate verification remain open.
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


## Phase 20G1 — pinned EngineBuilder adapter (implementation submitted)

The isolated Malachite spike now exposes a typed `build_engine` function that calls the real v0.8.2 `EngineBuilder` and supplies the upstream default WAL, network, consensus, sync and request actor configuration. Its generic bounds compile-check APEX's ability to pass upstream context, config and codec types through the public application-channel API. The adapter lives only in the isolated research workspace; it does not add Malachite to the production workspace.

This is an API integration boundary, not a running engine. No engine actors are started by the adapter's smoke tests; no validator key or network identity is created; no sockets, peer discovery, application event loop, APEX proposal codec, APEX finality verifier or durable application commit are integrated. The upstream test application uses its own test context, codec, signing and application semantics and must not be mistaken for APEX conformance.

The next step is a real, isolated multi-validator runtime using APEX-owned application validation and signer reservations, with independently provisioned test identities, process-level network fault injection, restart/recovery tests and certificate-to-application commit assertions. Only after that evidence should dependency promotion or production workspace integration be considered.
\n\n## Phase 20G2A — canonical consensus signing preimage\n\nThe apex-consensus::consensus_message module defines a fixed-width big-endian signing preimage with a dedicated APEX-CONSENSUS-MSG-V1 domain tag and encoding version. It binds chain ID, protocol version, height, round, consensus step, signer identity, validator-set hash and block hash. A zero block hash is accepted for nil prevote/precommit messages but rejected for proposals. Tests cover fixed width, domain separation, step/block binding, nil semantics and zero-context rejection.\n\nThis is not a signature implementation or certificate verifier. The profile is experimental and not ratified; no cryptographic suite or public-key identity mapping is selected. A real signature backend and strict certificate verifier are required before a Malachite runtime can rely on these bytes.\n