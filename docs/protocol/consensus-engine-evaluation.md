# Consensus Engine Evaluation — Phase 2

**Decision status:** Malachite is the preferred candidate for a bounded integration spike only. Production adoption is not approved.

## APEX requirements

APEX requires BFT proof-of-stake consensus with deterministic state-machine replication, explicit safety/liveness assumptions, validator-set changes, authenticated votes, commit certificates, crash recovery, bounded resource use, testable networking behavior and a maintainable upgrade path. Rust is the protocol implementation language.

## Candidates

### Malachite

Repository: https://github.com/circlefin/malachite

- Rust-native BFT consensus engine implementing Tendermint-style consensus.
- Upstream repository is maintained under Circle's organization; its README states that the Malachite team joined Circle and that Malachite is the consensus foundation for Arc.
- Upstream README describes formal specification/model checking work on parts of the Tendermint algorithm.
- **Current upstream workspace version: 0.8.2.**
- **Current upstream workspace MSRV: Rust 1.88.**
- Upstream explicitly labels Malachite alpha, under heavy development and not externally audited.

**Compatibility finding:** APEX currently declares Rust 1.85. Malachite's declared Rust 1.88 minimum is incompatible with the current APEX MSRV policy. No Malachite dependency has been added. APEX must decide whether to raise its MSRV, select an older upstream release (only if its support and security posture justify it), or reject the candidate. Do not bypass the MSRV declaration.

**Risks:** alpha maturity, API and release stability, audit gap, operational integration complexity, signer safety, persistence/recovery semantics and dependency-tree exposure. Arc's use is evidence of active engineering, not proof of APEX suitability or independent audit.

### CometBFT

Repository: https://github.com/cometbft/cometbft

- Go implementation and successor to Tendermint Core.
- Well-documented consensus and ABCI application boundary.
- ABCI++ provides deterministic application hooks for proposal preparation, proposal processing and vote extensions.
- Its documented safety model assumes less than one-third Byzantine voting power, subject to protocol assumptions.

**Risks:** Go runtime and operational component in a Rust-first node; integration requires a process boundary or interoperability layer. It remains a credible fallback/reference, but is less aligned with the selected Rust architecture.

## Current decision

Proceed with a dependency-free APEX adapter contract and integration plan. Do not add Malachite crates or claim that APEX has implemented BFT consensus until the MSRV decision and API review are complete. CometBFT remains the comparator if Malachite's compatibility or assurance gates cannot be met.

## Integration gates

1. Resolve APEX MSRV policy against upstream Rust 1.88; test the complete workspace at the chosen MSRV.
2. Pin exact Malachite release and source commit; record Apache-2.0 license, crate set, transitive dependency graph and reproducible lockfile.
3. Map engine lifecycle, application callbacks, proposal validation, commit notification, validator updates, signing, persistence, networking, metrics and shutdown/recovery.
4. Define signer safety and durable anti-double-sign state before connecting a validator key.
5. Reproduce upstream unit, model-based and integration tests available for the pinned release.
6. Add APEX-specific tests for conflicting proposals, equivocation, validator-set transitions, timeouts, partitions, delayed/reordered/dropped messages, crash/restart, state replay and deterministic application behavior.
7. Bound message sizes, queues, peer counts, validator input and storage growth.
8. Run multi-process fault injection and independent security review before public testnet use.
9. Document upgrade compatibility, rollback, state migration and incident response.

## Explicit non-decisions

No consensus dependency is currently in APEX Cargo manifests. No signature suite, chain ID, canonical serialization, validator economics, block interval, finality target or mainnet launch date is approved.

## Sources reviewed

- Malachite repository and README: https://github.com/circlefin/malachite
- Malachite workspace manifest (version and MSRV): https://github.com/circlefin/malachite/blob/main/code/Cargo.toml
- CometBFT repository: https://github.com/cometbft/cometbft
- CometBFT ABCI++ application requirements: https://github.com/cometbft/cometbft/blob/main/spec/abci/abci%2B%2B_app_requirements.md
- CometBFT ABCI++ concepts: https://github.com/cometbft/cometbft/blob/main/spec/abci/abci%2B%2B_basic_concepts.md
