# ADR-0058 — quiesced, read-only WAL snapshot acquisition

- **Status:** Proposed for review; isolated research implementation only.
- **Date:** 2026-11-08
- **Context:** ADR-0056 and ADR-0057 establish that the pinned Malachite default WAL open/fetch/startup paths can mutate evidence, while the byte-slice scanner does not acquire or authenticate the source bytes.
- **Decision:** Add a bounded read-only file reader that requires an explicit caller attestation of exclusive writer quiescence, reads through one opened file descriptor, compares metadata before and after the read, rejects detectable drift and length mismatches, hashes the captured bytes, and hands the snapshot to the existing scanner.
- **Limits:** The token is an operational assertion, not a lock. Metadata checks are race detection, not proof of an atomic snapshot; same-metadata rewrites and hostile storage behavior are not excluded. Operators must independently stop all writers for the full acquisition interval.
- **Non-decisions:** No OS-level lock, filesystem snapshot, authenticated provenance, anti-rollback anchor, backup durability proof, cross-store recovery manifest, custom WAL actor, reset authorization, signer wrapper, engine startup or production integration is approved.
- **Consequence:** The component creates a bounded byte snapshot under a documented quiescence precondition. It does not establish a trusted recovery authority. G2/G4 remain BLOCKED; release remains NO-GO.
