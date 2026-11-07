# ADR-0031: Fail-closed reopen validation for persistent EVM state

- **Status:** Proposed pending CI and review
- **Date:** 2026-11-07
- **Related:** ADR-0030, Phase 19F, Phase 19G

## Context

Phase 19F introduced a versioned redb backend for full-width EVM state. Its atomic publication model depends on a metadata schema version and current-state-version pointer. Reopening must not reinterpret incomplete metadata, silently select a supplied initial snapshot, or accept malformed current-version records.

## Decision

Keep the existing storage schema unchanged in this phase and strengthen the public reopen path with adversarial corruption tests. The adapter must fail closed for incomplete metadata, unsupported schema versions and malformed current-version account records.

No automatic repair is permitted. The caller-supplied initial snapshot is used only when the database is genuinely uninitialized; it is not a recovery fallback for damaged existing metadata.

## Consequences

- Reopen behavior becomes test-covered for selected corruption cases.
- Existing database bytes remain untouched by the adapter's recovery path when validation fails.
- Operators need a separately verified backup and documented manual recovery procedure.
- This does not prove behavior under actual process crashes, power loss, I/O faults or ambiguous commit acknowledgements.
- A future schema migration or recovery journal requires a separate ADR and explicit compatibility design.

## Rejected alternatives

- **Silently reinitialize on metadata error:** risks replacing valid or partially committed chain state with a different initial state.
- **Guess the current version from table contents:** introduces ambiguous recovery behavior and cannot prove which snapshot was committed.
- **Automatically repair malformed records:** risks fabricating consensus-critical state without a canonical commitment to validate against.
