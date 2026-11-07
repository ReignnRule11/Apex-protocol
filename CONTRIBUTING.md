# Contributing

APEX is in specification and foundation stage. Keep changes small, reviewable and evidence-backed.

Before a protocol change: read the specification and ADRs; explain threat-model and compatibility impact; add/update an ADR when a decision changes; update all affected protocol, architecture and security documents; include deterministic tests for consensus/execution changes; bind state-dependent validation to an explicitly identified parent-state snapshot; preserve transaction order and deterministic account-update order; enforce cumulative gas limits and reject duplicate temporary IDs; bind plans to a parent StateVersion; validate every before-state precondition and version before any write; require compare-and-swap and all-or-nothing application; keep storage adapters separate from protocol commitment encoding; keep the README tree synchronized; avoid unmeasured performance claims.

Persistent storage changes must commit all account updates and the next state version atomically, provide consistent snapshots, use explicit durability settings, fail closed on corruption, and pass reopen/recovery, concurrency, disk-full and fault-injection tests. Treat errors during commit as potentially indeterminate until reopened and reconciled. Never derive a consensus state root from database serialization, page checksums, or physical key order.

The workspace MSRV is Rust 1.90.0. CI and local toolchain configuration are pinned to this version. Do not lower or raise the MSRV without a reviewed ADR and full workspace/dependency validation.

Never commit secrets, validator keys, seed phrases or credentials. Do not silently change chain identifiers, encoding, monetary parameters or state-transition semantics. Incomplete behavior must fail closed rather than use permissive fallbacks.

Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`. Do not claim consensus security based solely on passing CI.
