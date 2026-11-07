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

## Transaction admission preflight (Phase 4)

The Rust consensus crate implements static checks over an internal transaction-admission structure: configured chain ID and protocol version must match; gas limit must be nonzero and no greater than a caller-supplied block gas limit; priority fee per gas must not exceed maximum fee per gas; and gas limit multiplied by maximum fee per gas must fit in u128. The helper returns that product as a maximum-fee upper bound, not a charged fee. No minimum fee is imposed because fee policy remains undecided. These checks do not verify signatures, sender identity, nonce against state, balances, intrinsic gas, calldata, canonical bytes, transaction ordering or execution. The structures are not a signed transaction type or wire format.

## Validity and current implementation boundary
The Rust consensus crate implements deterministic proposal-header preflight and limited static transaction-admission checks described above. These do not authenticate proposers, validate a complete block, decode canonical formats, verify commit certificates, execute state transitions or establish finality.

The protocol still requires rejection of malformed/non-canonical encoding, wrong chain ID, invalid signatures, nonce mismatch, insufficient fee balance, gas violations, invalid proposer/commit evidence and supply-accounting violations. Identical parent state and ordered transactions must produce identical receipts and state root. These full validity rules are not implemented.

## Validators
Lifecycle: bond, activation, participation, delayed exit and withdrawal. Separate consensus, staking and node-identity keys. Slashing only for objectively provable offences such as equivocation, with evidence and appeal rules specified before activation.

## Consensus assumptions and candidate status
The current research candidate is Malachite, a Rust-native Tendermint-style BFT engine; CometBFT remains a reference/fallback comparator. No engine is integrated, and production adoption is not approved. The upstream Malachite workspace currently declares Rust 1.88, while this APEX workspace declares Rust 1.85. This incompatibility must be resolved before dependency adoption. Upstream also describes Malachite as alpha and not externally audited. The common less-than-one-third Byzantine voting-power threshold remains protocol-dependent and is not an APEX security guarantee.

## Performance and compatibility
2–5 second blocks and 2–10 second finality are exploratory targets only, not measured results. EVM support requires a pinned fork and conformance tests for opcodes, precompiles, gas and edge cases.

## Open decisions
Consensus dependency and version, chain ID, codec, crypto suite, EVM revision, storage engine, validator economics, supply ceiling, issuance, fee parameters, governance thresholds and Bitcoin anchor policy remain undecided. No example value is an approved parameter.
