# ADR-0005: Canonical Protocol Encoding

- **Status:** Open
- **Decision:** No canonical wire encoding is selected yet. The existing codec crate must continue to fail closed.
- **Requirements:** Versioned schema; deterministic byte output; strict rejection of non-canonical encodings; explicit integer widths and endianness; bounded decoding; test vectors; domain separation for hashes/signatures; backward-compatibility policy.
- **Candidates to evaluate:** Deterministic CBOR with a narrowly specified profile; Protocol Buffers with strict deterministic encoding constraints; a purpose-built minimal codec. Compare canonicality guarantees, Rust library maturity, fuzzing surface, cross-language support and upgrade cost.
- **Rationale:** A codec choice affects transaction IDs, block hashes, state commitments and signature domains. Choosing casually creates consensus divergence risk.
- **Next action:** Build cross-implementation test vectors and fuzz malformed/truncated/oversized inputs before closing this ADR.
