# APEX Protocol Specification — Draft v0.1

**Status:** design draft; not production-ready.

## Purpose and goals
APEX is proposed as an independent Layer-1 infrastructure network, not a Bitcoin replacement. Goals: independent proof-of-stake BFT consensus; deterministic state transitions; Rust implementation; planned EVM-compatible execution; full transaction-data availability initially; optional Bitcoin checkpoint evidence; transparent monetary rules and constrained governance.

## Non-goals
No production mainnet, token sale, bridge, custom cryptography, novel consensus algorithm, or Bitcoin-equivalent security claim. No TPS or finality promise before reproducible benchmarks.

## Architecture
1. Consensus: validator set, proposal/voting, commit certificates, finality and evidence.
2. Execution: deterministic transaction validation and state transition.
3. State: canonical account/storage model and state root.
4. Data availability: blocks carry all transaction data.
5. Networking: authenticated peers, bounded decoding, quotas and peer scoring.
6. Anchoring: optional Bitcoin publication of APEX commitments.
7. Governance: delayed, transparent upgrades and bounded emergency powers.

## Transaction admission and planning (Phases 4–7)

The Rust prototype checks chain/version identity, nonzero gas limits, caller-supplied per-transaction and cumulative block gas bounds, fee-cap ordering and overflow-safe maximum-fee arithmetic. State-aware admission checks sender existence, exact nonce and conservative balance sufficiency against a supplied parent-state view. The sender is not authenticated. Pure transaction/block planners compute transfer and nonce updates against an in-memory overlay, preserve input order, merge writes, and fail the entire plan on any invalid transaction. Temporary TransactionId values are not canonical IDs or cryptographic hashes. Maximum fee is an escrow upper bound, not fee settlement. No signatures, EVM execution, actual gas, receipts or state root are implemented.

## Phase 8: versioned state application

The `apex-state` crate defines versioned immutable-by-interface snapshots, plans bound to a parent StateVersion, and a StateStore compare-and-swap contract. The in-memory reference adapter verifies version equality, unique ordered account updates and every before-state before writing any account. It applies all updates and increments the version once, or rejects without mutation. Replayed/stale plans fail. This is not durable or multi-process storage; StateVersion is not a state root or block height. Production persistence and crash recovery remain unimplemented.

## Validity and current implementation boundary

The Rust crates do not authenticate proposers or transaction senders, decode canonical formats, verify commit certificates, execute EVM state transitions, persist durable state, calculate state roots or establish finality. Identical parent state and ordered transactions must eventually produce identical receipts and state root; this is not yet implemented.

## Validators
Lifecycle: bond, activation, participation, delayed exit and withdrawal. Separate consensus, staking and node-identity keys. Slashing only for objectively provable offences such as equivocation, with evidence and appeal rules specified before activation.

## Consensus assumptions and candidate status
Malachite is a Rust-native Tendermint-style BFT research candidate; CometBFT remains a reference/fallback comparator. No engine is integrated. Malachite upstream declares Rust 1.88 while APEX declares Rust 1.85; this incompatibility must be resolved before dependency adoption. Upstream also describes Malachite as alpha and not externally audited. The common less-than-one-third Byzantine voting-power threshold remains protocol-dependent and is not an APEX security guarantee.

## Performance and compatibility
2–5 second blocks and 2–10 second finality are exploratory targets only, not measured results. EVM support requires a pinned fork and conformance tests for opcodes, precompiles, gas and edge cases.

## Open decisions
Consensus dependency and version, chain ID, codec, crypto suite, EVM revision, storage engine, validator economics, supply ceiling, issuance, fee parameters, governance thresholds and Bitcoin anchor policy remain undecided. No example value is an approved parameter.
