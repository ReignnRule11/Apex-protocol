# Consensus Design Notes

**Status:** engine candidate selected for research evaluation; no consensus engine is integrated or production-selected.

APEX intends to integrate a mature BFT proof-of-stake protocol rather than invent one. Malachite is the preferred Rust-native candidate for an integration spike; CometBFT remains the reference/fallback comparator. See [the engine evaluation](consensus-engine-evaluation.md), [ADR-0004](../architecture/adr/ADR-0004-consensus-engine.md), [the integration spike](consensus-integration-spike.md) and [the adapter contract](consensus-adapter-contract.md).

## Implemented preflight boundary

The `apex-consensus` crate currently checks a proposed header against an expected parent:
- chain ID must match;
- protocol version must match;
- height must equal parent height plus one, with overflow rejected;
- parent hash must match the expected parent block hash.

These checks are deterministic and fail closed. They are not canonical block decoding, cryptographic authentication, transaction validation, validator-set verification, commit-certificate verification or finality. The header types are internal Rust data structures, not a ratified wire format. See `validate_proposal_header` in `crates/apex-consensus`.

## Transaction-admission boundary

The crate also provides static transaction-admission checks; see transaction-admission.md. These validate only chain/version identity, gas-limit bounds, fee-cap ordering and overflow-safe maximum-fee arithmetic. The Phase 5 state-aware prototype also checks account existence, exact nonce equality and balance sufficiency against a caller-supplied read-only state view. That view must be a stable snapshot of the candidate parent state; this is a caller obligation, not enforced persistence or snapshot isolation. Sender identity remains unauthenticated, and these checks do not verify signatures, decode canonical bytes, execute transactions or establish complete validity. The block gas limit is caller-supplied, not an approved network parameter.

## Candidate constraints

The current APEX workspace declares Rust 1.85. The upstream Malachite workspace currently declares Rust 1.88 and version 0.8.2. Do not add the dependency until APEX's MSRV policy is deliberately updated and the exact crate set, license, dependency graph and API are reviewed. Upstream describes Malachite as alpha and not externally audited; neither active development nor another project's use is an APEX assurance case.

## Required protocol behavior

Specify proposer selection, vote phases, commit certificates, timeouts, validator-set transitions, equivocation evidence, restart recovery and partition behavior. The familiar less-than-one-third Byzantine threshold is protocol-dependent and is not a guarantee until the exact engine, validator weighting rules and synchrony assumptions are fixed.

## Integration boundary

The engine must not execute transactions or decide application validity on behalf of APEX. APEX's application adapter must validate proposals deterministically against the correct parent state, enforce chain and protocol versions, and commit only engine-finalized blocks whose application transition succeeds. Proposal-header preflight is only one early guard and cannot substitute for full application validation or engine certificate verification.

## Required adversarial tests

Conflicting proposals, equivocation, delayed/reordered/dropped messages, partitions, validator churn, restart/recovery, malformed certificates, long outages, resource exhaustion and deterministic replay. Add multi-process tests and fault injection before describing the network as a functioning testnet.
