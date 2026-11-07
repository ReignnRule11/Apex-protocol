# Contributing

APEX is in specification and foundation stage. Keep changes small, reviewable and evidence-backed.

Before a protocol change: read the specification and ADRs; explain threat-model and compatibility impact; add/update an ADR when a decision changes; update all affected protocol, architecture and security documents; include deterministic tests for consensus/execution changes; bind state-dependent validation to an explicitly identified parent-state snapshot; keep transition planning pure and require atomic all-or-nothing persistence in any future state adapter; document implemented boundaries and non-goals; keep the README repository tree synchronized with committed paths; avoid unmeasured performance claims.

Never commit secrets, validator keys, seed phrases or credentials. Do not silently change chain identifiers, encoding, monetary parameters or state-transition semantics. Incomplete behavior must fail closed rather than use permissive fallbacks.

Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` when the workspace is available. Do not claim consensus security based solely on passing CI.
