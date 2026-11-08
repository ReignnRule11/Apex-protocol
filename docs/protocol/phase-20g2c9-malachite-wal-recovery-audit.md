# Phase 20G2C9 — pinned Malachite WAL recovery capability audit

**Status:** Source audit submitted for review; no runtime changes or production approval.  
**Base main:** `35feaa2a3146aa9b7a4c1de430731bb4afedabd9`.  
**Pinned upstream:** Circlefin Malachite v0.8.2, commit `a453b32155f4ba9f959724279b52f601bae92e09`.  
**Scope:** Read-only audit of the exact pinned upstream source. No dependency revision, WAL behavior, signer, engine or production workspace is changed.

## Executive finding

The pinned release provides WAL replay/inspection primitives and explicit file synchronization, but its default recovery path is **not a non-destructive forensic/reconciliation interface**. Opening a WAL repairs a structurally incomplete tail by truncating and syncing it; fetching entries can truncate at the first raw read error; and the consensus actor resets the WAL on restart or when the node is not an active validator, while a sequence/height mismatch in the WAL actor also resets it. Those operations are incompatible with using the default startup path as the first step of APEX's cross-store recovery gate.

**Decision:** Do not pass an authoritative APEX WAL directly to the default `EngineBuilder::with_default_wal` startup path until an external preservation/preflight procedure and a guarded WAL integration have been designed and validated. A custom WAL actor is supported by the builder and is a candidate seam, but it is not yet proven sufficient for complete recovery evidence or non-destructive behavior.

## Audited source and version

All source claims below refer to immutable upstream commit `a453b32155f4ba9f959724279b52f601bae92e09`:

- [WAL engine actor](https://github.com/circlefin/malachite/blob/a453b32155f4ba9f959724279b52f601bae92e09/code/crates/engine/src/wal.rs)
- [WAL actor worker and replay fetch](https://github.com/circlefin/malachite/blob/a453b32155f4ba9f959724279b52f601bae92e09/code/crates/engine/src/wal/thread.rs)
- [WAL entry codec](https://github.com/circlefin/malachite/blob/a453b32155f4ba9f959724279b52f601bae92e09/code/crates/engine/src/wal/entry.rs)
- [Consensus startup, replay and signer/WAL effects](https://github.com/circlefin/malachite/blob/a453b32155f4ba9f959724279b52f601bae92e09/code/crates/engine/src/consensus.rs)
- [WAL log implementation](https://github.com/circlefin/malachite/blob/a453b32155f4ba9f959724279b52f601bae92e09/code/crates/wal/src/log.rs)
- [File-backed WAL and exclusive advisory lock](https://github.com/circlefin/malachite/blob/a453b32155f4ba9f959724279b52f601bae92e09/code/crates/wal/src/file.rs)
- [Storage contract](https://github.com/circlefin/malachite/blob/a453b32155f4ba9f959724279b52f601bae92e09/code/crates/wal/src/storage.rs)
- [Corruption tests](https://github.com/circlefin/malachite/blob/a453b32155f4ba9f959724279b52f601bae92e09/code/crates/wal/tests/it/corruption.rs)
- [Crash/fsync tests](https://github.com/circlefin/malachite/blob/a453b32155f4ba9f959724279b52f601bae92e09/code/crates/wal/tests/it/crashes.rs)
- [EngineBuilder and WAL actor injection](https://github.com/circlefin/malachite/blob/a453b32155f4ba9f959724279b52f601bae92e09/code/crates/app-channel/src/builder.rs)

## Findings

### 1. WAL record coverage is narrower than complete consensus/finality history

The engine WAL codec persists only signed votes, signed proposals, elapsed timeouts, proposed values and polka certificates. It explicitly rejects `StartHeight`, `Propose`, `RoundCertificate` and `SyncValueResponse`. Proposed-value origin is not retained; replay reconstructs it as `ValueOrigin::Consensus`.

The WAL header contains a format version and sequence number. It does not contain APEX chain ID, protocol version, validator-set activation identity, finality-journal tip or application-state commitment. The entry codec does not create a self-contained APEX finalized-block history.

**Implication:** the WAL is engine replay/equivocation evidence, not the canonical APEX finality ledger. It cannot independently prove that every APEX finalized height is represented or bind records to APEX chain/version without an external verified mapping.

### 2. Default open performs automatic tail repair

Pinned `malachitebft_wal::Log::open` validates the header and scans entry framing. If remaining bytes do not fit a complete final entry, it truncates the file to the last structurally complete position and calls `sync_all` before returning. This framing scan does not verify per-entry CRC; CRC validation occurs when entries are read through the iterator.

**Implication:** opening the only authoritative WAL may mutate evidence before APEX has copied, hashed and recorded it. Preflight must preserve an immutable byte-for-byte snapshot before invoking this open path. A copy alone is not a complete backup guarantee without trusted identity, provenance and anti-rollback controls.

### 3. Replay fetch truncates after a raw read error

The default engine worker's `fetch_entries` iterates raw WAL entries. On a raw iterator/read error, it records the error, calls `log.truncate(idx)` at that entry index, syncs through the WAL truncate implementation, then returns the collected entries plus the error.

A decoded-entry error follows a different route: it is retained as an error result in the replay vector; consensus replay emits a corruption event and enters its safety-failure/halt path. Thus raw read failure can mutate/truncate the WAL before the consensus actor sees the failure, whereas a codec decode failure is carried to the replay gate.

**Implication:** the default fetch API is not read-only. APEX must not treat post-fetch contents as the original forensic record. A custom preflight must classify raw framing/read/CRC failures without truncating and preserve all original bytes.

### 4. Consensus startup contains destructive reset paths

The consensus actor's height-start path calls WAL reset when `is_restart` is true, calls WAL reset for a node that is not an active validator, and otherwise fetches entries for the requested height. The WAL actor's `StartedHeight(height)` handler also compares the requested height with its WAL sequence; if they differ, it resets the WAL to the requested sequence and returns an empty replay list.

Reset updates the sequence, truncates all entries to the header and syncs the file.

**Implication:** neither default restart nor sequence mismatch is a safe implicit reconciliation mechanism. APEX must establish WAL sequence, finality tip, signer history and application marker before engine startup. Reset must be an explicit, separately authorized operational action after evidence preservation and review. A non-validator must not destroy validator recovery evidence in a shared data directory.

### 5. Append and flush are separate; signer invocation precedes WAL append

WAL `append` writes a framed record and updates in-memory entry count. WAL `flush` calls backing storage `sync_all` (the default file implementation calls `File::sync_all`). The consensus actor's normal publication path flushes the WAL before a signed message is broadcast, and routes append/flush failures to a safety-hang path.

However, `SignProposal` and `SignVote` call the configured signer first; the signed message is then WAL-appended; publication then flushes the WAL before broadcasting.

**Implication:** the WAL protects against broadcasting a message before its record is durable, but the signer is invoked before the WAL reservation exists. APEX's durable signer reservation is not called by the default engine signer path. A wrapper around Malachite's signer is a candidate integration point, but it must reserve exact native signing-domain bytes before delegating to the key signer. APEX's Phase 20G2A preimage is a distinct domain and cannot substitute for Malachite's native bytes. The exact canonical bytes and coordinate mapping must be proven from the pinned signer implementation.

### 6. Existing storage protections have limited scope

The default file-backed WAL obtains an exclusive advisory file lock; records include length and CRC framing; append failure truncates the partial append to its previous file position; and `flush` uses `sync_all`. Upstream tests cover selected corruption, truncation, process-termination and simulated sync-failure cases.

These primitives do not establish APEX cross-subsystem atomicity, backup anti-rollback, authenticated restore, chain identity, application-state reconstruction or the storage-device/filesystem guarantees APEX requires.

### 7. EngineBuilder provides a candidate seam, not a recovery guarantee

The pinned app-channel builder supports both `with_default_wal(WalContext { path, codec })` and `with_custom_wal(WalRef)`. This permits investigation of a custom WAL actor that enforces stricter policy. The custom actor must still satisfy Malachite's actor protocol. The builder API alone does not expose a complete WAL audit/recovery report, authenticated finality history or safe startup lock. APEX likely needs separate preflight/reconciliation before engine construction, and possibly a custom WAL actor to prevent reset/truncation paths from bypassing that gate.

## Capability matrix

| Required capability | Pinned upstream result | APEX decision |
|---|---|---|
| Enumerate framed WAL records | Available through `Log::iter` | Use only against a preserved snapshot or guarded reader |
| Decode engine replay inputs | Available through pinned `WalCodec` | Replay semantics only; not canonical APEX finality |
| Validate record CRC | Available during iteration | Preserve/report errors; do not accept silent truncation |
| Read sequence and format version | Public `sequence()` and `version()` | Compare against trusted APEX recovery manifest |
| Non-destructive open on damaged tail | Not provided by default open | Required APEX preflight/custom reader |
| Non-destructive raw read-error handling | Not provided by default fetch path | Required APEX preflight/custom reader |
| Prevent implicit height-mismatch reset | Default path resets | Must gate or replace default WAL actor |
| Prevent restart/non-validator reset | Consensus actor resets | Must control startup mode and engine integration |
| Durable append/flush boundary | Available, separate calls | Test exact ordering and ambiguous outcomes |
| APEX chain/version/finality tip binding | Not present in WAL header | External authenticated recovery manifest |
| Complete finalized-block history | Not present in persisted input subset | Finality journal remains separate authority |
| Signer reservation before key use | Not supplied by default signer path | APEX signer wrapper required; exact native bytes remain to be proven |
| Read-only authenticated recovery report | Not provided | APEX-owned component required |

## Required design before implementation

1. **Immutable preflight snapshot:** preserve/identify original WAL without opening it through the mutating default path; record path identity, byte length, content hash, format version, sequence and provenance. Define anti-rollback anchoring.
2. **Strict scanner:** implement/adapt a read-only scanner for exact pinned framing, length bounds, CRC, compression mode and entry codec. Report complete, partial, corrupt and undecodable records separately. Never truncate or repair source during audit.
3. **Explicit reset authorization:** prohibit default reset paths from operating on the only evidence copy. Decide whether a guarded custom WAL actor, disposable working copy or different engine startup design is required.
4. **Signer wrapper proof:** inspect pinned Malachite signing trait/encoding and prove the wrapper can derive exact bytes and coordinate before key use. Demonstrate reservation-before-sign, failure behavior and restart replay without double-signing. Do not equate APEX and Malachite signing domains.
5. **Cross-store manifest:** bind WAL snapshot identity/sequence, signer journal identity and monotonic watermark, finality tip, application commitment, chain ID, protocol version, engine revision and schema versions in an authenticated recovery report.
6. **Adversarial tests:** cover raw tail truncation, CRC corruption, codec failure, wrong sequence, restart flag, non-validator mode, reset failure, ambiguous append/flush, stale backup, copied WAL, lock contention and signer-wrapper failure. Assert no signing or consensus messages before the recovery gate opens.
7. **Independent review:** verify source audit, scanner, signer wrapper, storage assumptions and operator procedure before live engine wiring.

## Exit decision

**Default Malachite WAL is not approved as the sole APEX recovery authority or as the first component opened during recovery.** The pinned engine provides replay mechanics and durability primitives, but default open/fetch/startup paths can mutate or reset the WAL. APEX requires immutable preflight and a guarded startup design before integration.

This phase does not implement a scanner, custom WAL actor, signer wrapper, recovery manifest or runtime. The next implementation proposal must be separately scoped and tested. G2/G4 remain BLOCKED; production release remains NO-GO.

All conclusions are specific to pinned commit `a453b32155f4ba9f959724279b52f601bae92e09`; any Malachite revision change requires this audit to be repeated.
