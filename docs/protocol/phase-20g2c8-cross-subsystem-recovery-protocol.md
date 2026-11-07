# Phase 20G2C8 — cross-subsystem recovery protocol design

**Status:** Design specification submitted for review; no runtime integration or production approval.  
**Base main:** `6b258d05e889fd33650ab1336c03f19e11b7f23c`.  
**Preceding phase:** 20G2C7 merged; post-merge main CI workflow #42426724959 passed all four jobs.  
**Scope:** Design only. No Malachite WAL, signer journal, application database or finality journal behavior is changed by this phase.

## Objective

Define a fail-closed startup and commit protocol for reconciling four independently persisted domains without pretending they participate in one distributed atomic transaction:

1. Malachite consensus write-ahead log (WAL) and its replayable decisions.
2. APEX signer-safety reservation journal.
3. The APEX finality journal and its verified chain tip.
4. The materialized deterministic application state.

The protocol must prevent signing or consensus advancement until durable evidence has been reconciled. It must distinguish recoverable lag from rollback, conflicting history and evidence that cannot be inspected.

## Authority model

| Domain | Authority | Permitted recovery behavior |
|---|---|---|
| Consensus WAL | Evidence of engine-persisted consensus protocol state and replay position; not an application-state authority | Replay only through a version-pinned, inspectable adapter. A missing, unreadable or incompatible WAL is a startup blocker. |
| Signer reservation journal | Monotonic record of signing intents that must never be forgotten | Preserve every reservation. A reservation may legitimately be ahead of the WAL because reservation precedes signing. Never delete, rewind or reuse a reserved coordinate during recovery. |
| Finality journal | APEX's durable, ordered record of native-verified finalized block identities and the recovery data bound to each identity | Append only after native certificate verification and canonical identity binding. Exact retries are idempotent; conflicting identity/payload or broken parent linkage halts recovery. |
| Materialized application state | Rebuildable projection of the finalized journal, not an independent finality authority | Rebuild or replay deterministically when behind. Never accept an application database that is ahead of, or disagrees with, the finality journal. |

This is a proposed authority contract, not evidence that the current Malachite API exposes all required WAL records or replay metadata. An adapter must demonstrate those capabilities against the pinned upstream revision before implementation can proceed.

## Required ordering

### Normal finality path

1. **Reserve before signing.** Persist the exact canonical signing bytes for the active signer protocol/domain at its complete chain/protocol/height/round/step/signer coordinate before invoking a signer. An identical reservation is idempotent; different bytes at the same coordinate are a safety fault. APEX's Phase 20G2A preimage and Malachite's native signing bytes use different domains and must not be treated as interchangeable.
2. **Persist consensus protocol state.** The engine must durably record the relevant vote/decision state according to its documented WAL contract. APEX must not infer WAL durability from an in-memory channel message.
3. **Verify native finality.** Verify the native Malachite commit certificate with the pinned native verifier. Bind its value ID to canonical APEX block bytes, recomputed block identity, chain/version/height/parent and the authenticated validator context.
4. **Append finality record.** In one finality-journal transaction, append the verified identity and complete deterministic recovery payload, and advance the chain tip. A commit error is ambiguous; do not blindly retry a different payload or advance the engine.
5. **Materialize application state.** Apply the journal record to the application projection idempotently. Record the applied identity/height in the same application-state transaction as the state mutation, or make the projection fully disposable and deterministically rebuildable from the journal.
6. **Acknowledge finality.** Return success to the engine only after the finality journal and application projection agree on the exact identity. Only then may the event loop authorize the next-height transition.

The finality journal is the durable APEX application commit ledger in this design. The materialized application database is a projection. The design does not make the engine WAL, signer journal and finality journal one atomic database transaction.

## Startup recovery gate

Startup begins in **RECOVERY_LOCKED**. No validator signing, proposal/vote emission, application serving or consensus-height advancement is permitted until every required check succeeds.

1. Open all stores read-only where possible; validate schema/version, checksums/integrity metadata, chain identity and configured protocol version. Preserve the original stores for forensic inspection.
2. Validate signer reservations. Every WAL-observed signed vote/intent must map to an exact reservation through an explicit, versioned, signer-domain-aware coordinate mapping. Reservations not represented in the WAL are retained as abandoned-or-pending intents; they are not evidence of corruption by themselves. If the mapping or exact signing bytes cannot be proven, remain locked. A same-coordinate byte mismatch or a WAL signature without a reservation is a safety halt.
3. Read the finality journal's contiguous chain and tip. Validate each record's identity, parent linkage, payload bounds and canonical recovery-data format. Any gap, duplicate conflict, malformed payload or unexplained tip mismatch is a safety halt.
4. Replay the consensus WAL through an adapter that exposes enough authenticated, ordered decision evidence to compare it with the finality journal. Reverify each replayed finality certificate and canonical identity before considering it for append.
5. If the WAL contains a valid finalized decision immediately after the journal tip, append it through the normal idempotent finality path, in strict height order. Repeat until WAL and finality journal agree. A decision that cannot be independently reverified is not replay authority.
6. If the finality journal is ahead of the WAL, the WAL is missing/corrupt, or the adapter cannot establish a trustworthy ordering relationship, halt. Do not truncate the finality journal, rewind the signer journal, or synthesize consensus history automatically.
7. Compare the application projection with the finality journal. If it is behind, replay missing records deterministically and atomically. If it is at the same height but has a different state commitment/identity, or is ahead, halt. If it cannot be deterministically reconstructed from journal data, halt.
8. Require all stores to report the same chain ID and protocol version, and require the WAL's finalized position, journal tip and application applied marker to agree. Only then transition to **RECOVERY_COMPLETE** and permit the engine to start.

If a component cannot provide the evidence required by these steps, the correct behavior is to remain locked, not to infer success from a successful database open.

## Recovery decision matrix

| Observed condition | Required action |
|---|---|
| Exact finality record already exists; same identity and payload | Treat as idempotent; continue after all other checks pass |
| WAL has a later, valid, independently reverified finalized decision; journal is exactly one or more heights behind | Replay and append each missing height in order |
| Application projection is behind the journal and recovery data is valid | Deterministically replay; verify resulting commitment before unlocking |
| Reservation exists but corresponding WAL vote is absent | Preserve reservation; do not reuse coordinate; continue only if all WAL-observed signatures are covered |
| WAL contains a signed vote with no exact domain-aware reservation mapping | Safety halt; signing history cannot be proven safe |
| Same signer coordinate has different preimage bytes | Safety halt; preserve both evidence sources |
| Same height has a different finalized identity or payload | Safety halt; never overwrite or choose one automatically |
| Finality journal is ahead of WAL, has a gap, or has a parent mismatch | Safety halt; suspected rollback/corruption or unsupported recovery |
| Application state is ahead of or conflicts with journal | Safety halt; do not treat application state as authoritative |
| WAL replay cannot expose authenticated finality evidence or required order | Remain RECOVERY_LOCKED; adapter capability is insufficient |
| Commit result is ambiguous | Do not advance or blindly restart at the same height; stop and run the full recovery gate |
| Backup restore lacks a trusted anti-rollback anchor | Remain locked pending operator-approved recovery from independently authenticated evidence |

## Minimum adapter and data contracts before implementation

- A version-pinned Malachite WAL adapter must document durable write boundaries, replay ordering, finalized-decision extraction, corruption behavior and how native certificates/value IDs are recovered. If any item is unavailable, stop and reassess the engine integration.
- The signer adapter must expose immutable reservation enumeration and exact coordinate/bytes comparison without exporting private keys. It must identify the active signing domain and map Malachite's native signing coordinates/bytes to reservations explicitly; APEX's Phase 20G2A signing preimage is a distinct domain and cannot be substituted.
- Each finality record must contain canonical identity fields plus versioned, bounded, deterministic recovery data. The current journal's opaque payload is not sufficient until a canonical decoder, state commitment and replay semantics are specified.
- The application adapter must expose its applied identity and state commitment, and provide an idempotent transactional apply or deterministic rebuild path.
- All adapters must reject chain/version mismatch, non-contiguous heights, parent mismatch, duplicate conflicts, unknown schema versions and arithmetic overflow.
- Recovery logs must record evidence identifiers and decisions without logging private keys or secret material. Operator overrides must be explicit, authenticated, auditable and unable to silently rewrite finalized history.
- Backup rollback detection requires a separately protected monotonic anchor or independently authenticated checkpoint. The current repository has not implemented one.

## Failure containment and operational rules

- Recovery is serialized; concurrent startup/recovery writers are unsupported until separately designed and tested.
- Never enable validator keys, sign, vote, propose, serve state or advance consensus while RECOVERY_LOCKED.
- Never auto-delete, truncate, rewrite or downgrade any journal to make stores agree.
- An ambiguous commit, conflicting finality, missing signer reservation, unverifiable WAL decision or unexplained rollback requires a halted node and preserved evidence.
- Operator recovery must select an authenticated source of truth through a documented incident procedure; it must not be an unreviewed CLI flag or a best-effort database repair.
- A successful recovery run is not a safety proof. It must emit a machine-checkable reconciliation report tied to exact store identities and software versions.

## Required validation before any live integration

1. API-level contract tests for every adapter, including unsupported/missing evidence behavior.
2. Model-based tests covering every row of the recovery decision matrix and all permutations of crash points around reservation, WAL persistence, finality append, application projection and acknowledgement.
3. Multi-process exclusion and concurrent-open tests; explicit proof of single-writer enforcement.
4. Fault injection for process kill, host reboot/power interruption where feasible, disk-full, I/O errors, torn/corrupt records, truncated WAL, stale backups and restore rollback.
5. Deterministic state reconstruction and cross-version migration tests with canonical fixtures.
6. Byzantine/replay tests proving that a WAL record cannot bypass native certificate verification or canonical APEX identity binding.
7. Independent review of the recovery state machine, storage assumptions, operator controls and anti-rollback design.

## Explicit non-goals and current limits

- This phase does not implement the protocol, change code paths, start Malachite, open sockets, sign votes or enable validators.
- No claim is made that the pinned Malachite WAL exposes the required inspection/replay interface.
- No atomic transaction spans Malachite WAL, signer reservations, finality journal and application state.
- The current redb finality journal still stores opaque payload bytes and has only controlled process-termination tests, not power-loss or device-failure evidence.
- Signer backup rollback protection, authenticated snapshot restore, multi-process safety, production schema migration and independent audit remain open.
- G2 and G4 remain BLOCKED. Overall release remains NO-GO.

## Exit criteria

This design phase is complete only when the recovery authority model and decision matrix receive review. Implementation must be split into separately testable adapters and must not begin by wiring the existing prototype directly into the live engine. Any unavailable engine/WAL capability requires an explicit architecture decision before proceeding.
