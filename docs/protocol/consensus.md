# Consensus Design Notes

**Status:** Malachite v0.8.2 pinned as a research candidate in Phase 20A; no consensus engine is integrated or production-selected.

APEX intends to integrate a mature BFT proof-of-stake protocol rather than invent one. Malachite is the preferred Rust-native candidate for an isolated research spike; CometBFT remains the reference/fallback comparator.

## Implemented preflight boundary

The apex-consensus crate checks chain ID, protocol version, sequential height and parent hash. These are structural checks only; they do not authenticate proposers, verify certificates or establish finality.

## Transaction and block planning

The crate also provides static transaction-admission checks, state-aware nonce and affordability checks, and pure transaction/block transition planning. The block planner preserves caller order, observes earlier planned nonce/balance changes, merges recipient writes, rejects duplicate temporary IDs, enforces cumulative declared gas and returns no plan on any invalid transaction. Sender identity and temporary IDs remain unauthenticated.

## Versioned application-state boundary

The separate apex-state crate binds candidate block plans to a versioned parent snapshot and defines an atomic compare-and-swap application contract. Its in-memory reference adapter checks every before-state and version precondition before any write, rejects stale/replayed plans and increments the version only after all checks pass. It is process-local and non-durable; production storage must implement a serializable atomic transaction.

## Candidate constraints

APEX declares Rust 1.91.0. Malachite v0.8.2 declares Rust 1.88, so the current toolchain is compatible. Phase 20A pins the research baseline at upstream commit a453b32155f4ba9f959724279b52f601bae92e09 under Apache-2.0. No dependency is added: upstream labels Malachite alpha and not externally audited, and the exact integration crate set, feature flags, resolved dependency graph, API and signer/recovery design still require review.

## Required protocol behavior

Specify proposer selection, vote phases, commit certificates, timeouts, validator-set transitions, equivocation evidence, restart recovery and partition behavior. The familiar less-than-one-third Byzantine threshold is protocol-dependent and is not a guarantee until the exact engine, validator weighting rules and synchrony assumptions are fixed.

## Required adversarial tests

Conflicting proposals, equivocation, delayed/reordered/dropped messages, partitions, validator churn, restart/recovery, malformed certificates, long outages, resource exhaustion and deterministic replay. Add multi-process tests and fault injection before describing the network as a functioning testnet.
