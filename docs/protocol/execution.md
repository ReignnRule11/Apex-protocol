# Execution Design Notes

The proposed execution environment is EVM-compatible, backed by a maintained Rust implementation such as revm subject to an ADR and pinned revision. No EVM execution is implemented yet.

Before implementation define account/storage model, signing domain, nonce rules, gas schedule, fee settlement, contract creation, precompiles, fork activation, receipt encoding, state-root algorithm and error semantics. Demonstrate compatibility through upstream vectors, APEX-specific vectors, differential testing and fuzzing. Host time, randomness, network access and nondeterministic iteration must not affect execution.
