# Consensus Engine Evaluation — Phase 1

**Decision status:** Malachite is the preferred prototype candidate. Production adoption is conditional and is not approved by this evaluation alone.

## Requirements

APEX requires a BFT proof-of-stake consensus engine with deterministic state-machine replication, explicit safety/liveness assumptions, validator-set changes, authenticated votes, commit certificates, crash recovery, bounded resource use, testable networking behavior, and a maintainable upgrade path. Rust is the protocol implementation language.

## Candidates

### Malachite

Repository: https://github.com/circlefin/malachite

- Rust-native BFT consensus engine implementing Tendermint-style consensus.
- Repository documents design/specification work, including formal specification and model checking of parts of the protocol.
- The repository README identifies Malachite as alpha software, under heavy development, and not externally audited.
- The README states that the Malachite team joined Circle and that the engine serves as the consensus foundation for Arc.
- Strongest alignment with APEX's Rust implementation constraint.

**Risks:** alpha maturity, API and release stability, audit gap, operational integration complexity, and the need to verify exact crate versions and interfaces. Arc's use is relevant evidence of active engineering, not proof that APEX can safely adopt it or that the engine has been independently audited.

### CometBFT

Repository: https://github.com/cometbft/cometbft

- Go implementation and successor to Tendermint Core.
- Well-documented consensus and ABCI application boundary.
- ABCI++ provides deterministic application hooks for proposal preparation, proposal processing and vote extensions.
- Its documented safety model assumes less than one-third Byzantine voting power, subject to the protocol's stated assumptions.

**Risks:** Go runtime and operational component in a Rust-first node; integration would require a process boundary or additional interoperability layer. This is a credible fallback/reference implementation, but less aligned with the selected Rust architecture.

## Decision

Use Malachite as the preferred candidate for a contained research prototype and integration spike. Do not yet pin it as a production protocol dependency, expose a public validator API, or claim consensus security.

Keep CometBFT as the reference/fallback comparator. If Malachite cannot meet stability, safety-review, integration or operational requirements, reopen this decision rather than forcing adoption.

## Mandatory adoption gates

1. Pin an exact upstream release or commit and record license, dependency graph and maintenance status.
2. Review upstream protocol specifications and map APEX requirements to the engine's actual interfaces.
3. Reproduce upstream tests and add APEX-specific tests for validator-set transitions, equivocation evidence, timeouts, partitions, restart/recovery and state-machine determinism.
4. Run multi-process fault-injection tests with delayed, dropped, reordered and malicious messages.
5. Verify resource bounds for message sizes, queues, peer counts and untrusted validator input.
6. Obtain independent consensus/security review before any public network handles value.
7. Document upgrade compatibility and an operational rollback/recovery procedure.

## Explicit non-decisions

This ADR does not select validator signature algorithms, transaction signatures, chain identifiers, canonical serialization, validator economics, block interval, finality target or a mainnet launch date.

## Sources reviewed

- Malachite repository and README: https://github.com/circlefin/malachite
- CometBFT repository: https://github.com/cometbft/cometbft
- CometBFT ABCI++ application requirements: https://github.com/cometbft/cometbft/blob/main/spec/abci/abci%2B%2B_app_requirements.md
- CometBFT ABCI++ concepts: https://github.com/cometbft/cometbft/blob/main/spec/abci/abci%2B%2B_basic_concepts.md
