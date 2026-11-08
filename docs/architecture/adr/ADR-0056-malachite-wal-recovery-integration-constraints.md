# ADR-0056 — Malachite WAL recovery integration constraints

- **Status:** Proposed for review; no production adoption.
- **Date:** 2026-11-08
- **Context:** ADR-0055 requires WAL evidence to be reconciled without silently destroying history. Pinned Malachite v0.8.2 provides replay, CRC, sync and storage-lock primitives, but its default open/fetch/startup paths can truncate incomplete/unreadable records or reset a WAL based on restart/validator/height state.
- **Decision:** Do not use the default WAL open/fetch/startup path as the first operation on the sole authoritative APEX WAL. Require immutable preflight, strict read-only scanning and explicit reset authorization before engine construction. Treat the WAL as engine replay/equivocation evidence, not the APEX finality ledger. Investigate a guarded custom WAL actor and/or disposable working copy; neither is approved until it demonstrates actor compatibility and blocks implicit destructive paths.
- **Signer consequence:** The pinned consensus actor calls the configured signer before WAL append and flush-before-broadcast. APEX must investigate a signer wrapper that durably reserves exact Malachite-native signing bytes before key use. APEX's Phase 20G2A preimage is a separate signing domain and cannot be substituted.
- **Non-decisions:** No scanner, custom actor, signer wrapper, authenticated recovery manifest, anti-rollback anchor or runtime integration is implemented. Upstream API capabilities do not prove APEX cross-store atomicity or production durability.
- **Consequence:** Preserve the original WAL before any mutating open/fetch, fail closed on unexplained corruption or sequence mismatch, and keep G2/G4 BLOCKED. Production release remains NO-GO.
