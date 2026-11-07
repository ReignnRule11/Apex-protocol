# ADR-0051 — deterministic tests for Malachite channel finality gate

- **Status:** Accepted for isolated research validation only; not production adoption.
- **Date:** 2026-11-07
- **Context:** Phase 20G2C3 added a fail-closed gate over Malachite's actual `Channels::consensus` receiver, but only had a height-overflow unit test. Safety-critical behavior is the ordering between verification, staging, durable commit, acknowledgement and next-height instructions.
- **Decision:** Add deterministic tests using the exact pinned Malachite `AppMsg`, `CommitCertificate`, `Next` and Tokio receiver types. Extract the receiver loop used by `run_channels_finality_gate` so tests exercise the same receive path. Use the upstream test context crate only as an isolated dev-dependency; keep the production Cargo workspace unchanged.
- **Verified invariants:** Decided is acknowledged only after verification and staging and never commits or advances. Finalized sends `Next::Start(height + 1)` only after verification and commit. Verification, identity-height, commit and overflow failures cannot start the next height; restart failure drops the reply. The actual receiver processes queued messages before reporting closure.
- **Test isolation:** Test-only verified-token construction is guarded by `cfg(test)`. Validator keys, validator set and application callbacks are fixtures, not trusted production inputs.
- **Non-decisions:** No real native certificate verification, canonical APEX decoding, authenticated validator governance, durable database, atomic commit, signer anti-rollback, crash recovery, live engine, network fault testing or BFT safety/liveness proof is approved.
- **Consequence:** The tests validate control flow and channel plumbing, not application callback correctness. A concrete canonical-value/validator-provenance adapter and live multi-validator recovery testing remain mandatory before production consideration. G2 remains BLOCKED.
