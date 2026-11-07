# Consensus Design Notes

**Status:** engine candidate selected for research evaluation; no consensus engine is integrated or production-selected.

APEX intends to integrate a mature BFT proof-of-stake protocol rather than invent one. Malachite is the preferred Rust-native candidate for an integration spike; CometBFT remains the reference/fallback comparator. See the engine evaluation, ADR-0004, integration spike and adapter contract.

## Implemented preflight boundary

The `apex-consensus` crate checks chain ID, protocol version, sequential height and parent hash. These are structural checks only; they do not authenticate proposers, verify certificates or establish finality.

## Transaction and block planning

The crate also provides static transaction-admission checks, state-aware nonce and affordability checks, and pure transaction/block transition planning. The block planner preserves caller order, observes earlier planned nonce/balance changes, merges recipient writes, rejects duplicate temporary IDs, enforces cumulative declared gas and returns no plan on any invalid transaction. Sender identity and temporary IDs remain unauthenticated.

## Versioned application-state boundary (Phase 8)

The separate `apex-state` crate binds candidate block plans to a versioned parent snapshot and defines an atomic compare-and-swap application contract. Its in-memory reference adapter checks every before-state and version precondition before any write, rejects stale/replayed plans and increments the version only after all checks pass. It is process-local and non-durable; production storage must implement a serializable atomic transaction. See transaction-admission.md.

## Candidate constraints

The APEX workspace declares Rust 1.85. The upstream Malachite workspace currently declares Rust 1.88 and version 0.8.2. Do not add the dependency until APEX's MSRV policy is deliberately updated and the exact crate set, license, dependency graph and API are reviewed. Upstream describes Malachite as alpha and not externally audited; neither active development nor another project's use is an APEX assurance case.

## Required protocol behavior

Specify proposer selection, vote phases, commit certificates, timeouts, validator-set transitions, equivocation evidence, restart recovery and partition behavior. The familiar less-than-one-third Byzantine threshold is protocol-dependent and is not a guarantee until the exact engine, validator weighting rules and synchrony assumptions are fixed.

## Required adversarial tests

Conflicting proposals, equivocation, delayed/reordered/dropped messages, partitions, validator churn, restart/recovery, malformed certificates, long outages, resource exhaustion and deterministic replay. Add multi-process tests and fault injection before describing the network as a functioning testnet.
