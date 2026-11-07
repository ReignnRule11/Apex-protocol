# Consensus Design Notes

**Status:** engine candidate selected for research evaluation; no consensus engine is integrated or production-selected.

APEX intends to integrate a mature BFT proof-of-stake protocol rather than invent one. Malachite is the preferred Rust-native candidate for an integration spike; CometBFT remains the reference/fallback comparator. See [the engine evaluation](consensus-engine-evaluation.md), [ADR-0004](../architecture/adr/ADR-0004-consensus-engine.md) and [the integration spike](consensus-integration-spike.md).

## Candidate constraints

The current APEX workspace declares Rust 1.85. The upstream Malachite workspace currently declares Rust 1.88 and version 0.8.2. Do not add the dependency until APEX's MSRV policy is deliberately updated and the exact crate set, license, dependency graph and API are reviewed. Upstream describes Malachite as alpha and not externally audited; neither active development nor another project's use is an APEX assurance case.

## Required protocol behavior

Specify proposer selection, vote phases, commit certificates, timeouts, validator-set transitions, equivocation evidence, restart recovery and partition behavior. The familiar less-than-one-third Byzantine threshold is protocol-dependent and is not a guarantee until the exact engine, validator weighting rules and synchrony assumptions are fixed.

## Integration boundary

The engine must not execute transactions or decide application validity on behalf of APEX. APEX's application adapter must validate proposals deterministically against the correct parent state, enforce chain and protocol versions, and commit only engine-finalized blocks whose application transition succeeds. Engine callbacks, persistence, signing, validator-set updates and shutdown/recovery semantics must be explicitly mapped before integration.

## Required adversarial tests

Conflicting proposals, equivocation, delayed/reordered/dropped messages, partitions, validator churn, restart/recovery, malformed certificates, long outages, resource exhaustion and deterministic replay. Add multi-process tests and fault injection before describing the network as a functioning testnet.
