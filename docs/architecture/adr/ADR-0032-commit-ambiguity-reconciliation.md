# ADR-0032: Commit-ambiguity reconciliation for persistent EVM state

- **Status:** Proposed pending CI and review
- **Date:** 2026-11-07
- **Related:** ADR-0030, ADR-0031, Phases 19F–19H

## Context

The full-width EVM redb adapter writes a complete versioned snapshot and advances its current-version metadata pointer in one redb transaction. A commit error is ambiguous to the caller: it must not assume either that the transaction committed or that it did not.

## Decision

Retain the existing schema and atomic publication design. After any commit error, latch the adapter and require close/reopen reconciliation. The metadata current-version pointer remains the sole selector for the active snapshot; version-prefixed records without that pointer are not authoritative.

Add test-only fault injection for two distinct outcomes:
- a write transaction dropped before commit, leaving the prior committed version active;
- a successful redb commit followed by a simulated lost acknowledgement, requiring reopen to discover the advanced version.

Also test well-formed future-version records that exist without a corresponding metadata pointer. Reopen must not infer the active version from record presence.

## Consequences

- The tests make the ambiguous-commit recovery contract executable.
- A caller must reopen and inspect durable state before retrying after a commit error.
- A stale delta is rejected when the previous attempt actually committed.
- No production fault-injection switch or new public API is introduced.

## Rejected alternatives

- **Retry immediately after commit error:** unsafe because the first transaction may already be durable.
- **Select the highest version found in data tables:** unsafe because orphaned/incomplete records are not proof of publication.
- **Automatically repair or promote orphaned records:** unsafe without a canonical state commitment and explicit recovery protocol.

## Limitations

These tests do not simulate OS crash, power loss, filesystem faults, device errors, multi-process writers, or consensus/signer atomic commit. They are not an independent storage audit and do not establish production durability.
