# Contributing

APEX is in specification and foundation stage. Keep changes small, reviewable and evidence-backed.

Before a protocol change: read the specification and ADRs; explain threat-model and compatibility impact; add/update an ADR when a decision changes; update all affected protocol, architecture and security documents; include deterministic tests for consensus/execution changes; bind state-dependent validation to an explicitly identified parent-state snapshot; preserve transaction order and deterministic account-update order; enforce cumulative gas limits and reject duplicate temporary IDs; bind plans to a parent StateVersion; validate every before-state precondition and version before any write; require compare-and-swap and all-or-nothing application; document the in-memory adapter as non-durable; keep the README tree synchronized; avoid unmeasured performance claims.

For persistent storage work, keep backend integrity separate from protocol state commitments. A backend must atomically persist all account updates and the new state version, provide consistent snapshots, fail closed on corruption, and pass reopen/recovery, concurrency, disk-full and fault-injection tests. Never derive a consensus state root from database serialization, page checksums, or physical key order. Do not select a state commitment until canonical bytes, ordering, hash domains, tree semantics and test vectors are specified.

Never commit secrets, validator keys, seed phrases or credentials. Do not silently change chain identifiers, encoding, monetary parameters or state-transition semantics. Incomplete behavior must fail closed rather than use permissive fallbacks.

Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`. Do not claim consensus security based solely on passing CI.
