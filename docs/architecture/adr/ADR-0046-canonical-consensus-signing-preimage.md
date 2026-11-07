# ADR-0046: Experimental canonical consensus signing preimage

- **Status:** Accepted for isolated implementation only; not a ratified protocol decision.
- **Date:** 2026-11-07
- **Context:** Durable signer reservations require exact deterministic message bytes, but APEX has not selected a cryptographic signature suite or finalized its consensus wire protocol.
- **Decision:** Introduce a fixed-width, big-endian signing preimage with a dedicated domain tag and encoding version. Bind chain ID, protocol version, height, round, step, signer identity, validator-set hash and block hash. Permit nil block hashes only for prevote/precommit messages.
- **Consequences:** The encoding provides an APEX-owned preimage for future signer integration. Phase 20G2B adds an experimental Ed25519 verifier and authenticated validator-set hash, but the profile remains unratified and disconnected from Malachite/application commit. No canonical wire format or production finality is established.
- **Safety:** No production key or validator may use this experimental profile. G2 remains blocked pending runtime integration, authenticated validator-set governance, signer recovery, adversarial testing and independent review.
