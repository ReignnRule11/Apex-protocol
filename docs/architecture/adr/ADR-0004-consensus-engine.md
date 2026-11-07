# ADR-0004: Consensus Engine Candidate

- **Status:** Accepted for research evaluation and a bounded integration spike; production adoption conditional and not approved.
- **Decision:** Malachite is the preferred Rust-native BFT candidate for a contained prototype. CometBFT remains the reference/fallback comparator.
- **Context:** APEX requires BFT proof-of-stake consensus and Rust is the selected implementation language. CometBFT is a Go implementation; Malachite is Rust-native and Tendermint-style.
- **Rationale:** Malachite aligns with the Rust architecture and has evidence of active engineering. Its upstream README says the team joined Circle and that Malachite serves as the consensus foundation for Arc. The same README explicitly labels Malachite alpha, under heavy development and not externally audited. Arc's use is not proof of APEX suitability or independent assurance.
- **Compatibility finding (2026-11-07):** upstream Malachite workspace declares version 0.8.2 and Rust 1.88; APEX currently declares Rust 1.85. This is a direct MSRV mismatch. No Malachite crate is added to the APEX dependency graph.
- **Phase 3 implementation:** APEX now has an engine-neutral proposal-header preflight that checks chain ID, protocol version, sequential height and parent hash. It is not a consensus engine, certificate verifier or complete application validator. See [adapter contract](../../protocol/consensus-adapter-contract.md).
- **Consequences:** first resolve APEX MSRV policy, then pin exact upstream versions, audit APIs/license/dependency tree, map persistence/signing/network/application interfaces, reproduce upstream tests, run APEX-specific fault-injection and recovery tests, bound resources and obtain independent security review. Reopen this ADR if any gate fails.
- **Non-decisions:** No signature suite, chain ID, codec, validator economics, block interval, finality target or mainnet date is approved here.

## Evidence

- Upstream repository: https://github.com/circlefin/malachite
- Upstream workspace manifest: https://github.com/circlefin/malachite/blob/main/code/Cargo.toml
- Upstream README: https://github.com/circlefin/malachite/blob/main/README.md
- CometBFT reference: https://github.com/cometbft/cometbft
