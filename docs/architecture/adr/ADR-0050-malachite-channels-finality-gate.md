# ADR-0050 — Malachite Channels finality event-loop gate

- **Status:** Accepted for isolated research implementation only; not production adoption.
- **Date:** 2026-11-07
- **Context:** Phase 20G2C2 produced a native certificate verification and APEX identity-binding primitive, but did not connect it to Malachite's application channel. Malachite distinguishes `AppMsg::Decided` from `AppMsg::Finalized`; the latter requires a `Next` response.
- **Decision:** Add an isolated channel event-loop gate over the pinned `Channels::consensus` receiver. The application supplies native verification/value binding and storage callbacks. The gate stages and acknowledges `Decided` without advancing height. It re-verifies `Finalized`, calls the durable-commit callback, checks height increment, and only then sends `Next::Start`. Failures request `Next::Restart`; if restart parameters are unavailable, the reply is dropped and consensus stalls.
- **Ordering invariant:** No code path in this gate sends `Next::Start` for a `Decided` message. A `Finalized` message receives `Next::Start` only after verification and the application commit callback both succeed.
- **Delegation:** Non-finality messages are forwarded unchanged to the application's ordinary handler. The gate does not replace proposal construction, vote-extension policy, sync handling, networking or request processing.
- **Trust boundary:** The gate cannot prove that an application callback actually uses bounded canonical value retrieval, authenticated validator-set provenance, correct native verification or durable atomic storage. Those remain explicit implementation and review obligations.
- **Isolation:** The new direct engine dependency remains inside `experiments/malachite-spike`; the production Cargo workspace is unchanged.
- **Non-decisions:** No production consensus selection, validator governance, key custody, cross-system atomic commit protocol, restart recovery, network security or BFT safety/liveness claim is approved.
- **Consequence:** The next subphase must supply deterministic channel-level tests and a concrete APEX value/validator mapping implementation before a live multi-validator integration is attempted.
