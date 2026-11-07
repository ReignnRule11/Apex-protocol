# ADR-0049 — isolated Malachite-native finality adapter

- **Status:** Accepted for isolated research implementation only; not a ratified protocol or production adoption decision.
- **Date:** 2026-11-07
- **Context:** Phase 20G2C1 established that Malachite v0.8.2 commit certificates are signed in the engine's native signing domain and cannot be reinterpreted as APEX Phase 20G2A/20G2B signatures. APEX also requires a separate binding from the engine's value identifier to canonical application block identity.
- **Decision:** Add a generic adapter only to the isolated `experiments/malachite-spike` workspace. It checks native certificate height, round and value ID against the expected decision; delegates the native signature/quorum verification to the pinned `malachitebft_signing::VerifierExt::verify_commit_certificate`; requires an explicit application-supplied `ApexValueIdBinding`; and emits a private verified token only after all checks succeed.
- **Identity rule:** Malachite `value_id` is not assumed to be an APEX block hash. The application must strictly decode canonical APEX value bytes, recompute the APEX identity and provide a deterministic binding implementation. This adapter does not supply the codec or mapping algorithm.
- **Isolation:** The direct Malachite signing dependency remains outside the production Cargo workspace. No validator key, runtime actor, socket, mainnet or production finality is enabled.
- **Non-decisions:** No event-loop integration, application commit gate, durable atomic commit/recovery, authenticated validator governance, signer anti-rollback, production signature profile or BFT safety/liveness claim is approved.
- **Consequence:** The next subphase must attach this primitive to the real `Channels` event loop and prove that only a successfully committed `Finalized` event can advance application height.
