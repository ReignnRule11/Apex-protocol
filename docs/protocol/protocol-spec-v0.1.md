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

## Proposed transaction envelope
Protocol version, chain ID, sender nonce, recipient or contract-creation marker, value, calldata, gas limit, max fee per gas, max priority fee per gas, access list and signature. Canonical serialization and signing domain must be specified before clients are implemented. Signatures bind chain ID and transaction version to prevent replay.

## Proposed block commitment
Protocol version, chain ID, height, parent hash, timestamp, proposer, transaction root, state root, receipt root, validator-set hash and consensus commit certificate. Anchor metadata does not determine APEX consensus validity.

## Validity
Reject malformed/non-canonical encoding, wrong chain ID, invalid signature, nonce mismatch, insufficient fee balance, gas violations, invalid proposer/commit evidence and supply-accounting violations. Identical parent state and ordered transactions must produce identical receipts and state root.

## Validators
Lifecycle: bond, activation, participation, delayed exit and withdrawal. Separate consensus, staking and node-identity keys. Slashing only for objectively provable offences such as equivocation, with evidence and appeal rules specified before activation.

## Consensus assumptions and candidate status
The current research candidate is Malachite, a Rust-native Tendermint-style BFT engine; CometBFT remains a reference/fallback comparator. No engine is integrated, and production adoption is not approved. The upstream Malachite workspace currently declares Rust 1.88, while this APEX workspace declares Rust 1.85. This incompatibility must be resolved before dependency adoption. Upstream also describes Malachite as alpha and not externally audited. The common less-than-one-third Byzantine voting-power threshold remains protocol-dependent and is not an APEX security guarantee.

## Performance and compatibility
2–5 second blocks and 2–10 second finality are exploratory targets only, not measured results. EVM support requires a pinned fork and conformance tests for opcodes, precompiles, gas and edge cases.

## Open decisions
Consensus dependency and version, chain ID, codec, crypto suite, EVM revision, storage engine, validator economics, supply ceiling, issuance, fee parameters, governance thresholds and Bitcoin anchor policy remain undecided. No example value is an approved parameter.
