# ADR-0057 — strict read-only WAL preflight scanner

- **Status:** Proposed for review; isolated research implementation only.
- **Date:** 2026-11-08
- **Context:** ADR-0056 prohibits default Malachite WAL open/fetch/startup as the first operation on the only authoritative recovery evidence because those paths can truncate or reset records.
- **Decision:** Introduce an APEX-owned byte-slice scanner that parses the exact pinned v1 WAL framing without file I/O or mutation. It hashes the complete supplied snapshot, validates the header and bounded entry framing, verifies CRC32 for uncompressed records, and reports all conditions that prevent a fully verified result.
- **Compression policy:** Compressed records are reported as unverified. Their stored CRC is over decompressed bytes; framing alone cannot establish integrity. This phase deliberately does not claim compressed-record validation.
- **Non-decisions:** No snapshot acquisition, file locking, immutable copy, provenance, anti-rollback anchor, compressed decoding, engine payload decoding, authenticated recovery manifest, reset authorization, custom WAL actor, signer wrapper, engine startup or production integration is implemented.
- **Consequence:** A clean report means only that the supplied bytes have supported v1 framing and uncompressed CRCs. It does not prove a trusted snapshot, complete APEX history, finality, safe restart or durable storage. G2/G4 remain BLOCKED; release remains NO-GO.
