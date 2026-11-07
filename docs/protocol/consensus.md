# Consensus Design Notes

**No consensus engine is implemented or selected.** APEX intends to integrate a mature BFT proof-of-stake protocol rather than invent one. The selection ADR must compare production maturity, audit history, safety/liveness assumptions, validator churn, network synchrony, recovery, Rust integration, license and operational tooling.

Specify proposer selection, vote phases, commit certificates, timeouts, validator-set transitions, equivocation evidence, restart recovery and partition behavior. The familiar less-than-one-third Byzantine threshold is protocol-dependent and is not a guarantee until the engine and assumptions are fixed.

Required adversarial tests: conflicting proposals, equivocation, delayed messages, partitions, validator churn, restart/recovery, malformed certificates, long outages and deterministic replay.
