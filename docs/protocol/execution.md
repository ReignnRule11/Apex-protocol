# Execution Design Notes

The proposed execution environment is EVM-compatible, backed by a maintained Rust implementation such as revm subject to an ADR and pinned revision. No EVM execution is implemented yet.

## Implemented pre-execution boundary

The apex-consensus crate provides static transaction-admission preflight. It checks chain ID, protocol version, nonzero gas limit, gas limit not exceeding a caller-provided block gas limit, priority fee not exceeding maximum fee, and overflow-safe multiplication of gas limit by maximum fee. The returned product is a maximum-fee upper bound only; it is not charged, settled or checked against an account balance. No minimum fee is assumed.

This is not signature verification, sender authorization, account nonce validation, balance validation, intrinsic-gas calculation, calldata validation, canonical decoding or EVM execution. The input is an internal Rust structure, not a canonical transaction envelope.

Before execution implementation define account/storage model, signing domain, nonce rules, gas schedule, fee settlement, contract creation, precompiles, fork activation, receipt encoding, state-root algorithm and error semantics. Demonstrate compatibility through upstream vectors, APEX-specific vectors, differential testing and fuzzing. Host time, randomness, network access and nondeterministic iteration must not affect execution.
