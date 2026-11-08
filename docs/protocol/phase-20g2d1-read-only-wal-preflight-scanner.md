# Phase 20G2D1 — strict read-only Malachite WAL preflight scanner

**Status:** Implementation submitted for review; isolated research workspace only.  
**Base main:** `4ae3d22b352c7e12042b9564a452d43339050acd`.  
**Pinned upstream:** Malachite v0.8.2, commit `a453b32155f4ba9f959724279b52f601bae92e09`.

## Objective

Implement the first APEX-owned WAL preflight primitive required by Phase 20G2C9 without calling the upstream mutating `Log::open`, engine `fetch_entries`, or startup/reset paths.

## Implementation

Adds `experiments/malachite-spike/src/wal_preflight.rs` and exports it from the isolated spike crate.

- Accepts an immutable byte slice; it does not open a path, acquire a lock, write, truncate, repair, or sync storage.
- Computes SHA-256 over the complete input snapshot, including damaged trailing bytes.
- Parses the pinned v1 header (4-byte big-endian format version, 8-byte big-endian sequence).
- Parses each entry's compression flag, 8-byte big-endian payload length, 4-byte CRC32 and payload.
- Enforces the upstream 1 GiB entry bound and checked length arithmetic.
- Verifies IEEE CRC-32 for uncompressed payloads.
- Reports empty/incomplete headers, unsupported versions, partial entry headers/payloads, oversized entries and CRC mismatches without modifying the input.
- Compressed records are framed and scanning may continue, but they are explicitly marked unverified: upstream CRC covers decompressed bytes, and this phase does not add LZ4 decoding.
- Exposes entry offsets and a report-level `is_fully_verified` predicate. Any issue prevents a fully verified result.

## Test coverage

Deterministic unit tests cover empty and partial headers, header-only WAL, multiple valid uncompressed records, partial entry header, partial payload, CRC mismatch, oversized length, compressed-record fail-closed reporting, unsupported format version, and input immutability/hash retention.

## Security and scope limits

This is a byte-slice parser, not a safe file acquisition procedure. It does not prove that the supplied bytes are an immutable or authenticated snapshot, prevent concurrent mutation while the caller reads a file, establish provenance, anti-rollback, a trusted backup, or device-level durability. It does not decode Malachite engine entry payloads, verify compressed payload CRCs, recover consensus state, bind WAL sequence to APEX finality/signer/application stores, authorize reset, or integrate a custom WAL actor.

Do not pass an authoritative WAL path through default Malachite open/fetch/startup before preserving it. Do not enable validators or treat a clean framing report as proof of finality or safe restart.

**Exit:** scanner primitive and deterministic parser tests submitted. Next gates: exact-head CI, then review of safe snapshot acquisition and authenticated recovery manifest. G2/G4 remain BLOCKED; production release remains NO-GO.
