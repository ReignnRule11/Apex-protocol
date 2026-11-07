# ADR-0052 — idempotent finality commit outcomes

- **Status:** Accepted for isolated research validation only; not production adoption.
- **Date:** 2026-11-07
- **Context:** The Phase 20G2C3/C4 gate treated any `commit_finalized` error as a reason to request `Next::Restart(current_height)`. A database transaction may have committed before the callback lost its response or failed while constructing next-height parameters. Restarting without reconciliation can replay state. A conflicting identity at an already-finalized height is a safety fault, not an ordinary retry.
- **Decision:** Derive a stable `FinalityCommitKey` from the opaque verified APEX identity. Require the application to atomically persist state and this key, and make same-key retries idempotent. Classify commit outcomes as definitely not committed, ambiguous, or conflicting finality.
- **Engine response:** Only a proven `DefinitelyNotCommitted` outcome may request a same-height restart. `Ambiguous` and `ConflictingFinality` drop the Finalized reply so consensus cannot advance or blindly replay; durable recovery/operator intervention is required.
- **Invariants:** A commit key includes chain ID, protocol version, height, parent hash and block hash. It is derived from a verified token, not untrusted caller input. A same-key retry must return the existing committed result without reapplying state. A different key at the same height must fail closed.
- **Non-decisions:** No concrete database, atomic state/marker transaction, recovery journal, power-loss proof, signer anti-rollback, live engine or production adapter is selected or implemented.
- **Consequence:** The event-loop contract no longer conflates ambiguous commit outcomes with definite non-commit. Tests validate response classification only; G2 remains BLOCKED and release remains NO-GO.
